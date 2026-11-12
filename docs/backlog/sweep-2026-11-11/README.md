# The sweep of 11 November 2026: what was pruned, and where every row went

The appendix to [found-along-the-way.md](../found-along-the-way.md). Four pairs of helpers gathered what builders and reviewers had left open (609 rows) and checked each against the code and the branches in flight. The checked tables are beside this file: [checked-rules.md](checked-rules.md) (rows R: the rules pages, probes, designs and the project's documents), [checked-look.md](checked-look.md) (rows L: the look and the interface), [checked-core-reports.md](checked-core-reports.md) (rows C: the core's builders, reviewers and integrators) and [checked-code.md](checked-code.md) (rows K: what the code itself admits). Many rows cite reports and pictures under `.claude/night/`, which is the helpers' working folder on Stella's machine and is not in the repository.

## 3a. Everything pruned, with its reason

105 rows: 42 fixed, 32 moot (31 as checked, and R84, which goes with its twin C6), 31 not a fault. The reason is the checker's own, shortened where it ran long; the full text is in the checked table the letter names.

### Fixed: a later commit dealt with it (42)

| Row | What was reported | Why it is pruned |
|---|---|---|
| R23 | Carters are not counted in the employment series | The probe spoke of its prototype. The built mechanism counts them: carting.rs:112 count_carters, called at econ.rs:1087, adds carters to each town's employment (merged d71124f). |
| R52 | A small new town's wage can double monthly to the ceiling of 500; harvests move which seed has it | Under the game's rules: people.md ('As merged, no wage ran away any more'), prices and expectations go with their cells, carters by degrees (6470d78), tests/people.rs:224 plants a tiny town, examples/wage_watch.rs counts them. The port keeps it by design; what is left is item 54. |
| R55 | A new town's first year of land use is bid with 1750 base prices and a wage of nothing | towns::new_town_year (towns.rs:227-242, called at sim.rs:440; commit 2b2f59c) gives a new town its donor's year, wage and prices, before the land step. What is left is item 98. |
| R66 | Scripted players and the price shock were not run on prices, Engel and machines together | Both were run on the whole game at later merges: the scripted players at works.md:470, people.md:593 and towns.md:618, the shock at works.md:522. |
| R109 | Town cells that pay less than the open ground beside them were not checked cell by cell | The review's floor (1275290; towns.rs:187 and 438): a town cell pays at least the plough's rent, pits left out on purpose. examples/towns_edges.rs prints how often it binds (1 per cent of cell-years or fewer in company, towns.md:613). |
| R123 | Nothing tests the life cycle's switches | The probe's switches are gone. The built mechanism has tests/works.rs: eleven tests, among them off_is_the_port_and_on_is_the_game, the_years_room_at_its_edges and works_grow_where_they_stand. |
| R125 | Why engine-works capacity answers so slowly to a very high price was not traced | Traced in machines.md, 'Why machinery capacity answers so slowly' (a works with anything on order counts as building; entry of one works a year), and cured by the life cycle's first rule (works.md; d71124f). What took its place is item 112. |
| R127 | Untested follow-ups for towns: urban count from the year's mean not December's; seven towns in the 1750s empties the home town | Both done: the year's mean of works' hands (towns 'mean_hands', towns.rs:25, towns.md:108) and founding paced so the home town keeps its people (people.md:337). The rent floor too. |
| R128 | New town's land is priced as if labour were free for its founding year (the port and the Python) | Under the game's rules by towns::new_town_year (2b2f59c). The port and the Python keep it on purpose, since the port is held byte for byte; land_price works round it (FAMILIES.md). |
| R137 | Works squares record almost no ground rent | In the core, by kept town ground: towns::price_ground and rent_floor (towns.rs:486-497; 43efc01, merged 802b317), 'ground on town cells and on cells the works have taken is no longer nearly free' (towns.md:243). Nobody has looked at the rent view again. |
| L193 | Every works the player's branches open is a major news item | The chronicle, merged 4c543ee: replay.rs:92 major() goes by the event's `kept`, at most four a decade; the player's small items weigh 0.5 (chronicle.rs own_small). |
| L197 | Events carry no provenance: the test, threshold and value behind them | The chronicle, merged 4c543ee: every detected event carries `why` (docs/rules/chronicle.md, Provenance). |
| L198 | Events are not ranked per decade; the cut is a fixed weight in the viewer | The chronicle, merged 4c543ee: `kept`, per_decade 4 (docs/rules/chronicle.md, Ranking). |
| L199 | The observer tells no regional stories | The chronicle, merged 4c543ee: regional stories (coal boom, decline, fastest-growing town, closures). |
| L200 | A rival's works counts as its first of a kind again after it sells its only one | chronicle.rs:260 first_of_kind remembers what each family has ever held; families.rs:1142 (merged 4c543ee). |
| L203 | City milestones fire for the first town only | The chronicle, merged 4c543ee: 'Now every town has them' (docs/rules/chronicle.md, Three faults of the observer). |
| L204 | Doubling baselines are 1750 itself, with no check that it is typical | observer.rs:384 chronicle::baseline: the mean of the first five years (merged 4c543ee). |
| L212 | A failed railway subscription refunds landlords' money to households | Under the game's rules ways.rs:487 repay() returns what is left to each subscriber by what it put in (merged d71124f); the port keeps the Python's way on purpose. |
| L213 | Railway projects buy with no cash check; funding never stops a started line | Under the game's rules a line buys only what its cash pays for and makes calls (ways.rs:830-880, merged d71124f). What remains is in docs/rules/railways.md: calls have no limit, nothing abandons a line. |
| L214 | Lines promoted in the same year do not see each other and can both pay for the same cells | docs/rules/railways.md: 'Lines building count as laid ... two lines of one year never pay twice for cells they share' (merged d71124f). |
| C1 | The shock test (`prices_probe --shock`) was to be run on the merged rules and the review never could | Run when works joined the game: `docs/rules/works.md` 521-531 and `prices.md` 471-477 (commit 860cc0c, output in `core-steps/works-probe`). The shortage is no longer; the rebuilding overshoots and rings, which the rules pages carry as the building cycle. Not run again after railways, carting, people or towns. |
| C8 | After a loss of capacity the trough is as deep as before, and iron output falls further than the capacity lost | Run again at the works merge (`works.md` 521-531, commit 860cc0c): capacity is back a year sooner and real output loses a quarter as much. What is left is the overshoot and ring, the building cycle of the rules pages. Why iron output fell further than the capacity lost was never traced. |
| C28 | The viewer was built, not run, with works of 100 usual sizes, several to a cell, and a new refusal in the site list | The works generators draw sizes 0.4 to 100 (`crates/isostate-gen/README.md` 231; 3ca7d92, 9f5459f, merged 59a3b3b), `scatter.rs` says what a crowded cell does (viewer README 191; 4ba8f44, 7f9f317), and the standard shots were taken on the merged game (3d5d430). The refusal reaches the site list as the core's own words (`families.rs` 1086); nobody has said how it reads on screen. |
| C30 | The converter is not built once steel is cheap: a content change, not a rules change | With the machines' content beside the works rules the converter is built on all eight seeds: `first_output_bessemer` 1851 [1826 .. 1876], `share_bessemer_1880` 0.835 (`merged-game-f6-f` 206, 219; `works.md` 414; merge a5c43a5). It now comes too soon on some islands: item 15. |
| C32 | A newcomer's land bid on a cell the works have mostly taken is still the port's floor of 0.01 | `towns::open_rent` and `towns::price_ground` (`towns.rs` 172, 490; commit 3c42694, merged de0a6d3): a cell the works have mostly taken costs what it would earn as open land (`towns.md` 132-137). |
| C42 | A town can hire more hands than it has (a "wild year"); employment is not rationed to the people there are | Not by rationing, which is still not built, but at the causes found: prices and what people expect go with a district (`peopling.rs` 711-730, `prices::found_town`), carters are taken on by degrees (6470d78), a new town's year is its donor's (`towns::new_town_year`). `core-steps/people-probe/wage-watch-game.txt`, seeds 1 to 16: no town-month over 4 times the middle wage, the greatest 2.3; `tests/people.rs` holds seed ... |
| C43 | Two wild years that are not foundings (1825, 1836) were not opened | As 42. `carting.md`, 'The wage that flipped', found the cause in the railway years: carters hired all at once on last month's bill. |
| C44 | A new town's first year of land use is bid with base prices and a zero wage | `towns::new_town_year` (`towns.rs` 230), hooked in `sim::yearly` after either founding rule (2b2f59c; `towns.md` 111-119). What it left, the donor's prices taken whole, is the rules pages' (Fenmere). |
| C46 | The scripted players and the shock test were not run on the mechanisms together | The scripted players were run at the works, railways and carting, people and towns merges (`works.md` 470, `carting.md` 264, `people.md` 593, `towns.md` 618) and the shock at the works merge (860cc0c). |
| C62 | The runaway wage of a few seeds survives the merchants' prices; cause not traced | As 42: seed 11's greatest town wage is 1.9 times the middle town's (`wage-watch-game.txt`). |
| C80 | The chronicle's thresholds were set before the works life cycle and the population rule | A district's closures are a story at five works (ca5e7f6), and declines and fastest-growing towns were read again under the people rules with towns redrawn by their cells (e780736). The 10 per cent margins were left. |
| C81 | The chronicle's redrawn districts should be checked against how the towns mechanism moves catchments | e780736: `redrawn_towns` reads `st.pp_moved` against `REDRAWN_SHARE`, and `tests/chronicle.rs` holds a town to it. |
| C90 | Out of sample with carting the rails price peak is less tame than the test asked | In company the dearest year for rails is 1.58 times (0.76 to 4.11) on twelve seeds and 1.61 [0.62 .. 2.73] in the merged game: the steel rail and mills that can grow (`railways.md`, 'In company'; merge df1bcee). |
| C98 | Machinery capacity grows slowly however dear machinery is; never traced | Traced in `machines.md` 254-277 (a296fa8): an order shrinks geometrically, so a works counted as building for six years and could not grow. The works life cycle changed that line (a5c43a5). What still holds a small trade back is the year's room, item 27. |
| C101 | One or two towns fewer by 1900 under the life cycle | The people rules found towns all century: 18.9 [14 .. 21] in 1900 in the merged game (merge 3ad1e81). The thin early count is item 127. |
| C107 | People should move with their cells whenever catchments change, not only at a founding; ports and junctions as town sites not tried | `peopling::move_with_cells`, every year (57e9279, merged 3ad1e81). A rule for stations or junctions was declined (`people.md` 711-716). |
| C109 | Lines promoted together pay twice for the cells they share | `ways.rs` 20 and 314: lines building count as laid, and a line lays only its own cells (686e36d). |
| C112 | Land rent in a town's founding year reaches absurd sums, and families price land on it | As 44 (`towns.md` 111-119). |
| C116 | Whether single outer town cells rent below the farms they displaced was not checked | `rent_floor`: a town cell pays at least the plough's rent (`towns::plough_rent`; 1275290); it binds on 1 per cent of cell-years (`towns-probe/edges-game.txt`). |
| C117 | Under kept ground the 1750 map has about a third more town cells | The first towns' ground is back-dated and fitted: 61.6 cells in 1750 in the merged game for 61 before (`towns.md` 83-89; 3c42694). |
| C120 | Seven towns founded in the 1750s empty the home town | People rules: two foundings in the 1750s on seed 7, and Melbourne has 19,306 people in 1780 for 14,243 (`people.md` 337-343; merge 3ad1e81). |
| C122 | The town's size is taken from December's works, not the year's | `towns::month` and `year_hands` (`mean_hands`; 3c42694). |

### Moot: what it was about is gone, or is being replaced (32)

| Row | What was reported | Why it is pruned |
|---|---|---|
| R4 | The fact growth_capped_* counts a famine year at the cap as capped | facts.rs:241-242 is unchanged, but the cap it measures is the port's alone: under the game's rules the people mechanism replaced it (people.md; fact town_growth_near_old_cap). Only the port's baseline row is a shade off. |
| R64 | With machines the largest works are smaller; never traced | Measured under the port's size limit of four usual sizes. The life cycle for works replaced that rule (merged d71124f): the middling works is now 5.5 usual sizes and the largest 119. |
| R65 | One island ends with derelict ground; never opened | Seen before the life cycle for works. The merged game ends with 0 to 5 derelict cells on eight seeds (merged-game-f6-f facts.csv), where one seed had 45. |
| R96 | Railway towns were held back: thin markets, and without limits two seeds stop on a money error never traced | The probe's hack was replaced by the people mechanism, which ran the crash down (people.md 'What the probe's crash was'; tests/people.rs:224). That there is still no rule for railway towns is item 74. |
| R97 | With the works life cycle the railway is a tenth shorter and there are two or three towns fewer | Measured with the port's promoters and the port's founding of towns (971 km for 1,039). Both were replaced when railways and people were merged (d71124f): 2,600 km and towns founded to 1840. |
| R118 | The viewer was not looked at with the life cycle: tiers, plots, the works list and the new site refusal | The tiers and plots it worried about are gone: the works generators merged at 59a3b3b draw a works at any size from 0.4 to about 100 and take ground by the room it needs, and night/works-detail is rebuilding them again. What is left (the works list and the refusal line) is added below as an item of its own. |
| R126 | The wage cost of late towns is measured and not explained | The probe's hack was replaced by the people mechanism, whose page accounts for the cost (the found_saving table; the founding test counts the district's saving). What is left is items 68 and 70. |
| R131 | Whether three changed pixels are worth the ground and sea colours being in the look file | The baseline was taken again at the interface merge (DEVLOG: 'the set to compare with from here on') and five later commits added to look.toml (house kit, far view, hill towns). There is no three-pixel difference left to weigh. |
| L34 | Leaning wood edges make painted fields smaller than the record says | The lean is gone on night/wood-edges (fields::FRINGE removed in 5fa17a7; 8e8ab60: 'arable is cut by a wood's edge too and keeps its area'). Measure the new rule instead. |
| L43 | Shader and CPU disagree on some pixels away from a class edge in the hill view, not looked into | The edge rule those pixels came from is replaced on night/wood-edges (5fa17a7); task C requires ISOSTATE_COMPARE_CLASSES to be run on the new rule. |
| L61 | A townhouse's era is found by scanning the years on every rebuild, uncached | For a game, night/town-ground c54de1f dates each plot from the record (kept.rs, Story::year_of); the old scan stays only for runs under the port's rules (houses_port). |
| L66 | The night's baseline shots are no longer like-for-like after the simulation and the top bar changed | Later pieces compare against sets shot again after each merge (look3.js REF = shots/merged-8-houses-far), not against shots/baseline. |
| L87 | Several works sheets were never opened: forge, crucible, millwright, charcoal burning, other, states-great, most close and far pages | night/works-detail deef8f7 rebuilds every works on the kit; those sheets are redrawn and its reviewer renders them. |
| L150 | At 1x the Build and Buy buttons go unlit for a moment near each year's end | The yearly stepping it describes is replaced on night/calendar (88f594b: a month at a time, mid-year orders wait); look again there, where buttons still go unlit while the year turns (game.rs:766). |
| L154 | The fix for a decision opened while the year turns was caught only late in the century | The turn is reworked on night/calendar (game.rs there) and calendar.js asks its proof to cover 'a year's turn' and 'an order given mid-year landing at the turn'. |
| L155 | In the first year after a start, orders cannot be given while the clock runs | night/calendar asks for a month at a time from a first guess (clock::MONTH_GUESS, TURN_GUESS), and orders wait instead of being refused. |
| L182 | Claypit and iron-mine ground never appears on island 7, and derelict ground hardly at all | night/mines-review (8b2abeb) records each cell's workings by sector for the viewer, in place of diggings read off the land classes. |
| L189 | first_built rescans every year for each high-street plot: cost grows with the year | As row 61: for a game night/town-ground reads a plot's year from the record; first_built stays only for runs under the port's rules. |
| L210 | A spur site refused once is never tried again | The port's planner only (rail.rs:214). Under the game's rules the promoters of ways.rs replace rail::plan_spurs and a refused line cools off for a few years (docs/rules/railways.md), merged d71124f. |
| C6 | More famines without families under the merchants' prices, not traced | The famines opened (seed 13, 1853-58) were the machinery famine, which the machines mechanism has since replaced (`machines.md`: machinery's peak 12.4 times usual down to 3.5), and a game is played with families. Famines in the game as it stands are items 49 and 130. |
| C14 | Seed 6: the player's own family on autopilot makes far less news and has fewer births, not traced | What it counted was mostly a branch's works opening one by one. The chronicle (merge eb4cdaf) folds those into a yearly summary and the works life cycle leaves a family a few dozen works: `news_family` is 404 [381 .. 441] on eight seeds in the merged game. The integrator read it as one island's luck. |
| C21 | Works are smaller at the top under the machines, untraced | The works life cycle replaced the port's size limit: ninth tenth 33, largest 119 usual sizes in the merged game (`merged-game-f6-f` 150-152). |
| C25 | A field-by-field save must carry `machine_stock` | A save is the seed, the rules and the log of orders, played again on loading (`save.rs` 1-8, commit e97cea0); no field of the state is written. |
| C45 | One island of six ends with 45 derelict cells in 1900, not opened | Under the life cycle a closed works leaves ground derelict only at a quarter of a cell (`works::leaves_derelict`); merged game `cells_derelict_1900` 1.6 [0 .. 5]. |
| C57 | The farm zigzag had little room left under its bar of 1.2 once the curve joined prices | The harvests, merged since, take the zigzag to about 2 by weather (`harvest.md`, 'In company': 0.60 off, 1.95 on); the bar of 1.2 was for an island without weather. See 126. |
| C59 | The railway of 1900 came out shorter with the curve on one set of seeds; not sure it is noise | Measured under the port's promoters (about 1,100 km). The game's railways replaced them (merge df1bcee; 2,600 km in 1900) and nobody has set the curve on and off under them. |
| C70 | A save must carry the nine `harvest_*` fields and the harvest's random stream | As 25: a loaded game plays the years again from the seed, so it draws the same weather. |
| C85 | Records, replays and saves with the new railway fields unverified; `WayBook` does not derive Serialize | A save plays the orders again (e97cea0), so `WayBook` needs no Serialize; records and replays of the game's railways have been drawn by the viewer since (standard shots on the merged game, 3d5d430). |
| C97 | The `stock_thin` variant of the glut brake, for towns with no outflow, was never tested | `stock_thin` was the probe's prototype and is not in the code. The brake that was built counts sales and shipments (`prices.rs` 51-55); its own form of the question is item 9. |
| K21 | A works that finds no ground stands on the cell's highest land and overlaps its neighbours | `scatter.rs:114` to 136 and 377 to 381. The whole of "where the works stand" is about to be replaced by `night/works-sites` (queued behind `night/works-detail`; brief `look3.js:100` item 6: "plots that fit the ground instead of a grid of rectangles ... never on ground too steep or in a river"). The brief does not say what happens when a cell has more works than room, so the count must be carried to that branch's ... |
| K23 | A works in a crowded cell is drawn smaller than its size | `scatter.rs:128` to 130 (`Slots::firm_cap`). The same allocator, about to be replaced by `night/works-sites` (brief item 6), and `night/mines-review` gives mines little surface, which frees room. The brief does not name the crowded cell, so carry the count to its review: 540 of 13,328 works drawn under size (`art-integrate.md:88`). |
| R84 | More famines with prices when there are no families | Checked as open ('not looked at since'), but it is the same fault as C6, whose checker opened the famines: they were the machinery famine, which the machines mechanism has since replaced, and a game is played with families. Pruned with C6. Famines in the game as it stands are in piece 18. |

### Not a fault: a decision already taken, or it cannot happen (31)

| Row | What was reported | Why it is pruned |
|---|---|---|
| R9 | Eight seeds is a small sample for the sweep's star and up/down tally; wages and swings need sixteen | A caution on how to read a sweep, not a defect, and already the practice: every review ran seeds 9 to 16 and the calibration brief (core-continue.js, Calibrate) asks for sixteen. |
| R189 | The second island generator lacks fields the tree's later stages need: wind by cell, fish, harbour, salt | Settled in the design's revision (production-tree.md 16, row 11; 17.1 item 4): wind, fish and harbour are derived from what the generator has, and salt is dropped. Building them is stage 2's work, not a gap in the generator. |
| L129 | Demo games being left unsaved is the reviewer's reading, not the builder's | A decision taken and written down: README, Saved games ('--demo games ... neither continue the autosave nor write it'). One condition in live::start if Stella wants otherwise. |
| L134 | A good's price line is jagged from year to year | The reporter's own finding: the line draws the record as it is. How prices move is the prices rule's business (merged with the game's rules). |
| L135 | The Map views button reopens the last view rather than starting at Rent | Deliberate and in README, Map views ('brings back the view last used'). |
| L143 | A chart would not gain new years while the clock is wound back | It cannot arise: game.rs drive (394-411) asks for a year only when the clock is at the newest one. |
| L209 | The catchment byte would wrap past 254 towns | It cannot happen while T_max is 48 (record.rs:237); a note for whoever raises T_max. |
| L211 | The spur nearness test is not column distance | rail.rs:193 reproduces the Python as written, on purpose (PORTING.md); the game's rules do not use it. |
| C58 | Nobody has seen how the viewer's charts take the nine new series | The charts take five series by name (`ui.rs` 1074-1080) and a good's price and output by name; a series they do not name is not drawn and does no harm. Whether to show the chained wage is 53. |
| C88 | The game's branch lines do not visibly pay the country back | Under the rules the game is played with it comes out the other way by 1900: 3.5 per cent more baskets a head, nine seeds of twelve (`railways.md` 288-300, commit 044d9a8). Nothing shows in 1850. |
| C96 | With `machines.inherit = 0` in the whole game there is no converter but a higher wage | Decided at the works merge: left at 0.5, 'since a game without a converter is the poorer for it and the wage is not in danger' (`machines.md` 354-358). The tuning is item 15. |
| C102 | Shops fall to a few dozen large "works" (bears on the shop fronts left to Stella's taste) | A decision (`works.md` 148-156): a size limit for craft trades was measured and cost a wave of closures, so a 'works' of shops is a street of them. Drawing shop fronts is in BACKLOG under 'Waiting on Stella'. |
| C104 | Utilisation ignores the merchant's spread | Tried and left as the port has it, on measurement (`works.md` 157-164). |
| C108 | The parity tests probably pass without running: the Python fixtures are not in the repository | By design: the fixtures are 0.5 to 0.8 GB and not committed; a missing one prints SKIPPED and `ISOSTATE_REQUIRE_PARITY=1` makes it a failure (`tests/common/mod.rs` 91-103). `tests/golden.rs` holds the port. |
| C111 | Railway towns were put on hold: thin markets, and two seeds stop on a money error never traced | A decision: the probe itself said hold, and `people.md` 711-716 declines a rule for stations. The money error was the prototype's with its limits off; none of it is in the merged code. |
| C125 | Merged game: grain's price swings harder than under the port | The game's swing is the weather's: 0.027 with harvests off and 0.100 on (`harvest.md`, 'In company'). The port's 0.090 was the false cycle the merchants' prices removed. What may be too much is the echo: 69 and 126. |
| C133 | Merged game: the first rolling mill and engine works come later | `towns.md` 581-599: before, an island's first engine works was founded in a new town's first year at a wage of nothing; `donor_wage` ended that, and the discoveries are no later. The rest of the gap to the port is the year's room, item 27. |
| C136 | Merged game: 1750 is not the port's 1750 | The game's 1750 has carters, an eighth of all hands (`carting.md` 335-342), and a warm-up under its own rules (`prices.md` 486-491): those mechanisms' decision. What follows from it is 54 and 92. |
| K7 | The DERELICT land class carries a dead_code allowance | `world.rs:33` to 34: the constant keeps the class numbering whole beside `terrain.wesl:164`, 895 and 1172, where the shader draws class 11, and the allowance states that reason. |
| K8 | The paint tables' `NAMES` carries a dead_code allowance | `look.rs:73` to 74: read by the tests at `look.rs:567` to 568 that hold the shader's uniform fields to the look file's order; the allowance is there because only tests read it. |
| K9 | The core's shared test helpers allow dead code wholesale | `tests/common/mod.rs:2`, `tests/common/dump.rs:2`: each test binary compiles the shared module and uses part of it, which is what the allowance is for in Rust; it hides nothing in the library. |
| K12 | A recorded Python run stops lining up with the port after some years | `tests/parity.rs:969` to 978 and `PORTING.md` ("as expected for a model that amplifies rounding"): a bisection's rounding grows tenfold every two or three years until a near-tie falls the other way; the yearly step from Python's own state of 1786 matches exactly. |
| K13 | A chronicle test was loosened when kept town ground was merged | `tests/chronicle.rs:610` to 629 against `chronicle.rs:1195` to 1203: `chronicle::rank` keeps one place a decade for the family's heaviest item by design, and the test now says so (commit 5b28792). It states the rule; it does not hide a failure. |
| K14 | A hamlet on a tiny market area stays poor, and a test holds that as acceptable | `tests/people.rs:211` to 222 and `docs/rules/people.md:616` to 622 and 728 to 730: the hamlets are planted through the test hook `st.pp_found_next` on sites the founding rule refuses (`the_probes_district_is_refused`), to see that nothing runs away; no normal run makes one. |
| K17 | Embedding a run in the HTML viewer is done by the Python only | `PORTING.md:340` to 341 lists it as a deliberate deviation ("HTML embedding is not ported: `python3 -m isostate html` reads Rust `run.bin` files"), and the Bevy viewer is the game's. |
| K18 | The family parameters do not follow the convention later mechanisms follow | `params.rs:63` to 65 says they "stay as they are": the `family_*` fields at 202 to 245 are flat, written to params.json only when changed, and a save names them; renaming would break both for no gain. |
| K20 | Boats are decoration too | `boats.rs:1` to 2 and the viewer README line 192 ("Life"): boats stand off harbour towns, more for a bigger town, which the record does know. When canals and lake boats land, drawing those is ready-next 1. |
| K27 | Runs recorded before the land mix fall back to one class a cell | `replay.rs:229` and 237 to 244: both the Rust recorder (`record.rs:19`) and the Python's write the three mix layers, so the fallback only keeps old recordings playable. |
| K28 | `--look X,Z` is still taken as a point to look at | `main.rs:299` to 304, kept on purpose with a test at `main.rs:2350` to 2352; no brief, script or document uses the old form any more, so it costs nothing and breaks nothing. |
| K30 | The interface's colours are not in the look file | A decision of the look file's brief, `artlab-continue.js:92`: "Interface colours (ui.rs, family.rs, game.rs, icons.rs) stay where they are"; `look.rs:18` records it. |
| K42 | MISSED. The spur planner's nearness test is not a column distance | `rail.rs:193` to 198: "Python's nearness test, reproduced as written". Under the game's rules `rail::plan_spurs` returns at once (`rail.rs:166` to 168; `params.rs:507` switches the game's railways on), so it runs only in the port, where doing what the Python does is the point. The same thing is item 211 of `gathered-look.md`. |

## 3b. Where two checkers looked at the same fault and came away differently

| Rows | What each said | What this list does |
|---|---|---|
| R84 open; C6 moot | More famines under the merchants' prices when there are no families. | Pruned with C6 (see 3a). |
| R54 open; C42, C43, C62 fixed | A town can hire more hands than it has. The rationing rule was never built; the causes of the wild years were mended one by one and a watch on sixteen seeds finds no town's wage above 2.3 times the middle town's. | The symptom is pruned as fixed. R54 stays, among the smaller things, as the unbuilt rule only. |
| R3, K11 open; C108 not a fault | The parity tests skip without the Python fixtures. | Skipping is by design (C108). What stays open, low, is that nobody has run the comparison since the fixtures were made (R3, K11). |
| R137 fixed; L130 open | Works ground records almost no rent. | Fixed in the core; L130 stays in piece 2 as 'look at the rent map on today's build first'. |
| R118 moot, C28 fixed; R203 open | The viewer was not looked at with the works life cycle. | The tiers and plots are gone; what is left, the works list and the refusal line on screen, is R203 in piece 25. |
| C104 not a fault; R122, L20 open | Shops are about eighty large works. | The rule is a decision taken; drawing shop fronts is already under 'Waiting on Stella'. |
| C88 not a fault; R94 open | The branch lines do not visibly pay the country back. | In the game they do by 1900 (C88). R94 stays in piece 18 only for the hurdle and induced traffic, which are still the port's numbers. |
| C125 not a fault; R121 open | Grain swings harder than under the port. | The swing is the weather the harvests put back (C125). R121 stays for the peak in output per head and the alarms. |
| C133 not a fault; R107 open | The first engine works comes later. | Its lateness since the last merge is an accident removed (C133). R107 stays in piece 18 for the works that does not pay for thirty years after it is known. |
| C111 not a fault, R96 moot; R74 open | Railway towns. | The probe's hack and its money error are gone. R74 stays in piece 19 as a rule that builder and reviewer declined to invent. |
| R153 in flight; L51 open | Nobody has watched the houses change level in a window. | The scripted check is with night/winter-fixes; the measurement is in piece 14; a person's eye is still wanted. |
| K21, K23 moot; L71, C118 in flight; L72 open | Works drawn under size, or with no ground, in a crowded cell. | The allocator is being replaced by night/works-sites, whose brief does not say what a crowded cell does. The two counts (540 and 19 of 13,328) are to be asked of its review: piece 8. |
| R65, C44 and R64, C21 | Derelict ground on one island; works smaller at the top. | Both pairs agree: moot. |

## 3c. What has moved on the branches since the checkers looked

The checkers read the branches as they stood earlier in the evening. Read again for this list (git log only, nothing built):

- `night/winter-fixes` is at 7c2b3b3: `world::class_share` and `tools/compare_classes.py` are gone (L46, K1, K35), and the camera's swap of houses was checked in a live game at every speed (L53, R153). Its commit 3d885fb also says 'trains run on across a new year', which is half of L165 in piece 14; boats are not named.
- `night/wood-edges` is at 2d08134: the small closes follow the towns as they stand each year (L44).
- `night/works-detail` is at c813b21: every generator is in the registry and on the scale, and bridges have three levels. Nothing yet names L57 or L62.
- `night/accounts` is at 92e4904, `night/land-review` at 3933317 (its page written after review), `night/smooth-probe` at e265753 (the recorder can now try two cures). No report from the probe yet.
- The two scratch branches of C71 are still there.
- Files that no branch in flight touches, which is what 'at once' rests on: `prices.rs`, `harvest.rs`, `peopling.rs` and `tests/engel.rs` in the core; `trains.rs` and `cursor.rs` in the viewer; `PLAN.md`, `README.md` and `FAMILIES.md`. Every other file named in Part 1 has at least one branch in it (from `git diff --name-only rust-3d...BRANCH` for the thirteen branches).

## 3d. Where every open and in-flight row went

So that the gathering can be checked: each of the 504 rows kept is under exactly one heading. 'Faults' counts a fault once however many rows tell it; a fault told both by an open row and by a row in flight is counted where its open row stands.

| Where | Rows | Faults | The rows |
|---|---|---|---|
| 1. A real game is not lost to a test start | 10 | 10 | C138 C139 L49 L120 L121 L122 L124 L126 L127 L128 |
| 2. Map views and charts that say what is so | 13 | 10 | L35 L115 L119 L130 L137 L138 L139 L140 L141 L142 R135 R136 R138 |
| 3. Carts, roads and bridges (Ready next 2) | 6 | 5 | K19 K38 L101 L109 L110 L167 |
| 4. A works that closes stands empty (Ready next 6) | 3 | 2 | L73 R140 R151 |
| 5. Towns, once drawn from kept ground | 6 | 6 | L54 L58 L60 L64 L180 R100 |
| 6. Railways, once the new profiles are in | 10 | 8 | C110 L10 L98 L104 L178 L184 L186 L217 R90 R91 |
| 7. What the core pieces add (Ready next 1) | 6 | 5 | K24 L41 L93 L100 L185 R158 |
| 8. Works, once on the kit and laid out | 9 | 8 | L69 L72 L80 L81 L82 L89 R101 R148 R187 |
| 9. What the record carries, and a test | 10 | 8 | K5 L16 L131 L132 L146 L192 L196 R2 R104 R157 |
| 10. A save that outlives a change | 5 | 2 | K16 L12 L13 R30 R139 |
| 11. The sweep's tools (first step of Ready next 5) | 10 | 9 | C38 C51 C126 C137 R6 R7 R8 R11 R194 R195 |
| 12. Traces with the books of account | 17 | 13 | C10 C18 C66 C86 C87 C93 C134 C140 K39 K40 L206 R15 R21 R45 R76 R81 R102 |
| 13. A lab that shows what the game runs | 7 | 4 | K31 K32 L25 L27 L112 R129 R130 |
| 14. The smooth hand-over (Ready next 4) | 20 | 18 | L22 L28 L40 L51 L52 L90 L91 L92 L144 L145 L153 L164 L165 L171 L172 L173 L183 L208 R159 R162 |
| 15. Carting and roads that cost (Ready next 7) | 9 | 8 | C91 C113 C114 L216 R12 R14 R17 R18 R19 |
| 16. A railway company (Ready next 7) | 7 | 4 | C82 C94 K15 L215 R86 R87 R88 |
| 17. What the simulation knows and the picture does not show | 8 | 8 | C24 C65 K4 L6 L166 L175 L224 R164 |
| 18. The pass over all the rules (Ready next 5) | 51 | 32 | C3 C7 C15 C16 C17 C20 C27 C34 C36 C39 C49 C50 C54 C68 C83 C89 C92 C100 C123 C124 C128 C129 C130 C132 C135 R13 R16 R36 R38 R39 R56 R57 R58 R62 R68 R69 R77 R79 R89 R94 R99 R103 R107 R111 R112 R113 R116 R117 R119 R120 R121 |
| 19. New towns, and districts that change market | 9 | 8 | C106 C127 R70 R71 R72 R73 R74 R98 R105 |
| 20. A harvest month | 9 | 6 | C67 C69 R46 R47 R48 R49 R50 R51 R197 |
| 21. The tree's later stages (with Ready next 8 and 9) | 13 | 12 | C22 C105 K33 R59 R60 R161 R181 R183 R184 R185 R188 R190 R191 |
| 22. The run to 2050 (Ready next 10) | 3 | 3 | R75 R160 R168 |
| 23. Documents that still describe the port | 10 | 7 | C26 C35 C41 C52 C95 K36 K37 L222 R10 R202 |
| 24. The clock, the cards and the present | 10 | 8 | L114 L118 L133 L152 L156 L158 L160 L162 R133 R134 |
| 25. The news, watched in the running game | 18 | 12 | C29 C72 C74 C76 C77 C78 C131 L205 R24 R25 R27 R28 R32 R33 R34 R35 R198 R203 |
| 26. Winter and the light | 8 | 6 | K25 L5 L42 L95 L99 L169 L170 L181 |
| 27. A title screen | 3 | 3 | L11 L125 L223 |
| 28. Life and weather | 6 | 4 | L1 L2 L9 L30 L88 R163 |
| Smaller: guards and tests in the core | 16 | 11 | C2 C5 C12 C13 C56 C64 C79 C99 K11 R1 R3 R29 R83 R85 R108 R199 |
| Smaller: loose ends in the rules | 19 | 16 | C9 C19 C33 C63 L201 L207 L218 L219 R41 R42 R44 R53 R54 R61 R63 R80 R92 R167 R196 |
| Smaller: the land, after night/wood-edges | 3 | 3 | K26 L33 L50 |
| Smaller: the viewer's code | 12 | 12 | C48 K2 K3 K6 K29 L45 L147 L157 L168 L176 L177 L190 |
| Smaller: the interface's toolkit | 2 | 1 | K10 L23 |
| Smaller: tools and housekeeping | 4 | 4 | C47 C71 L116 L174 |
| Smaller: wants someone at the window | 7 | 7 | L55 L97 L111 L113 L123 L136 L159 |
| Waiting on Stella: the core loop | 25 | 19 | C4 C31 C37 C75 C103 C115 R20 R26 R43 R67 R82 R110 R114 R115 R124 R165 R166 R169 R170 R171 R172 R173 R174 R175 R176 |
| Waiting on Stella: decisions and matters of taste | 37 | 26 | C11 C53 C60 C61 C73 L20 L21 L29 L37 L56 L65 L70 L77 L78 L84 L85 L96 L102 L103 L105 L106 L107 L108 L151 L163 R22 R31 R40 R78 R122 R141 R142 R143 R144 R145 R146 R147 |
| In flight: night/mines-review | 3 | 1 | C40 L36 R5 |
| In flight: night/canals-review | 2 | 2 | C84 R95 |
| In flight: night/land-review | 1 | 1 | L221 |
| In flight: night/accounts | 3 | 3 | L194 L195 R177 |
| In flight: night/tree | 15 | 13 | C23 C55 C121 L202 L220 R37 R178 R179 R180 R182 R186 R192 R193 R200 R201 |
| In flight: night/calendar | 4 | 3 | L117 L148 L149 R132 |
| In flight: night/rails | 5 | 4 | L8 L76 L179 R93 R149 |
| In flight: night/town-ground | 10 | 8 | C119 K41 L18 L63 L161 L187 L188 L191 R106 R152 |
| In flight: night/works-detail | 8 | 7 | L17 L19 L26 L57 L59 L62 L79 R154 |
| In flight: night/works-sites (queued) | 13 | 8 | C118 K22 K34 L7 L24 L67 L68 L71 L74 L75 L83 L86 R150 |
| In flight: night/wood-edges | 9 | 8 | L14 L15 L31 L32 L38 L39 L44 L48 R156 |
| In flight: night/winter-fixes | 10 | 5 | K1 K35 L3 L4 L46 L47 L53 L94 R153 R155 |
| **All** | **504** | **391** | |

## 3e. The same fault told in more than one place

90 groups, which take 203 rows down to 90 faults. Every source is kept: the rows of a group are cited together in Part 1.

| Rows | The fault |
|---|---|
| R3 K11 | Parity tests check nothing without the Python dumps, which are not in the repository |
| R5 C40 | Quarries, pits and mines take a fifth of the land by area; cause never traced |
| R10 C51 C52 | Engel's new facts are not in the README's table of facts and parity/check_facts.py does not recompute them |
| R13 C92 | PLAN's targets for 1750 to 1800 were set without carting, which makes the early economy smaller |
| R14 C91 | The freight rate never moves while the money wage rises, so a cartload takes fewer hands by 1900 with no better road |
| R15 C87 | Three per cent handling on all trade is on the carting bill and no road or railway lowers it; its share of the bill unmeasured |
| R21 C86 | Railways raise output but not the real wage; where the gain goes was not traced |
| R26 C75 | The player's place in the history is nearly always a succession; works ordered by hand weigh less |
| R27 C78 | Markets news weighs every good alike and reads oddly for goods barely made |
| R28 C72 C74 | In a live game a kept news mark can be taken back; the viewer side (feed, ticks, toasts) was never looked at on screen |
| R31 C73 | A works the player orders by hand may get no toast |
| R33 C77 | A district's coal boom can be partly a railway redrawing which pits are whose |
| R34 C29 C131 | Single closures are told too often: the threshold for a notable closure is the port's |
| R38 C49 | With the Engel curve grain is jumpier and famines about double |
| R39 C54 | Engel's y_ref is tied to 1750 incomes; anything that lifts 1750 above 2 bundles changes the start |
| R40 C53 | News, gdp_real and the charts still use the 1750 basket, not the chained index |
| R46 R48 C69 | Grain's price saws for more than one year after a harvest shock |
| R53 C63 | Very large harvest shocks slow the port's issue of money; not traced |
| R56 C16 | Machine quality reaches its cap by 1900 and growth has moved into the first half of the century |
| R57 C15 | The converter's numbers and the quality curve were never tuned again with the works life cycle and families |
| R58 C17 C129 | Steel rails do not win under the life cycle |
| R59 C22 | A steel rail's longer life is credited at the mill, so it cheapens new track as well as relaying |
| R60 C23 | Steel has no use of its own; households take nine tenths of implements as steel |
| R63 C19 | Nobody buys a machine outright; with adopt_pumps on machinery's share could pass the whole price |
| R70 C106 | The town-founding test counts the district's carting saving, not the island's |
| R76 C18 | Extreme growth settings fail the money check; not traced |
| R78 C11 | The price rule's flow term cannot see exports, so an exporting town reads as a glut |
| R79 C7 | Rails and machinery prices still swing |
| R80 C9 | A town whose only buyer of a good goes away keeps its last stock and its works stop dead |
| R81 C10 | The glut brake is on works only; farms, pits, quarries and woods sell merchants everything |
| R82 C4 | Money buys more in 1750 under the price rule, so the families' first purses are worth half as much again and gdp_real cannot be set beside the port's |
| R83 C5 | glut_months depends on two numbers it does not own |
| R85 C2 C99 | Never looked at under the price rule: the commons rent regime, years after 1900, the Markets panel showing the anchor, the Python |
| R87 R88 C82 K15 | Railway calls have no limit: nothing asks whether a line is still worth building when it overruns |
| R89 C89 C135 | The date and size of the railway age varies widely between islands |
| R90 C110 L217 | Every cell of track is a station |
| R99 C128 | Town ground is taken whole from the plough, costing farm land and likely some of the real wage |
| R110 C115 | Families cannot buy town ground, and there is far more of it round a seat; effect on the family game not traced |
| R112 C27 | A trade far too small for its market grows only at the room's pace: tools famines and dear early machinery |
| R113 C39 | Prices swing harder under the life cycle, most with families |
| R114 C37 | A player who founds a works every year loses most of them, and has no way to extend a works |
| R115 C31 | A family's works are not rationed by the year's room |
| R116 C132 | Utilisation is lower under the life cycle |
| R119 C34 | The life cycle's numbers were checked only as a set |
| R121 C36 C100 C123 C124 | Output per head peaks before the end of the century, downturn alarms are above the port's and grain swings more |
| R122 L20 | Shops are about eighty large works; shops on every town cell would have to be drawn from the cell's services capacity |
| R124 C103 | AI families' own choices were never examined under the life cycle |
| R129 L27 | The lab has no live sliders or size tokens |
| R130 L25 K32 | The lab's scenes.rs repeats the towns' layout rules by hand and will drift from scatter/towns.rs |
| R132 L117 | Postings and stances given while a year turns land a year late |
| R133 L118 | Town labels show through cards |
| R134 L114 | The end card does not cover the speed buttons |
| R135 L119 | The price map view's scale is flattened by one outlying town |
| R136 L115 | The map overlay's inks are not in the look file |
| R138 L137 | A price exists for goods nobody makes yet |
| R139 L12 L13 K16 | A saved game does not survive a change to the simulation, and loading replays every year |
| R140 R151 L7 | Idle and derelict works are drawn in the lab but the simulation does not record them |
| R141 R142 L85 | Taste: the size of a family's pennant |
| R143 L102 | Taste: a church moves once, when a railway is begun through its yard, and stays moved |
| R144 L5 L106 | Taste: winter woods are thin brown sticks at play distance and autumn is one strong orange |
| R145 L96 | Taste: a lowland town before 1815 is one warm orange-red field |
| R146 L108 | Taste: a hamlet's church is as big as a market town's |
| R147 L21 | Taste: villas follow the year built, not the simulation's rents |
| R148 L69 | Small works of some kinds are hard to tell apart at play distance |
| R149 L76 | Works yards are placed as if the railway were straight, so a curving line can pass through one |
| R150 L86 | Works yards drape over steep ground |
| R152 L18 | Dense towns are narrow three-storey houses standing apart |
| R153 L51 L53 | Nobody has watched the houses' level of detail change in a window |
| R154 L17 | The works have no levels of detail |
| R155 L3 | No snow on roofs |
| R156 L14 | From about 8 km up a wood's outline follows parcel sides and reads as steps |
| R157 L16 | The record keeps two land classes a cell where the picture would use four |
| R158 L36 | Works ground and quarries are still the old rule's blocks on the map |
| R159 L22 | A year's scenery rebuild is a visible hitch |
| R162 L28 | Visible change (a staggered ladder of steps inside the year) and scenery built per chunk with a cache are not done |
| R163 L1 L2 | Not started for the look: construction dust, people on the roads, weather, a title screen |
| R187 K34 | Not designed for the tree: the families' lists for new trades, the viewer's silhouettes and ground textures, the observer's news for new goods |
| R197 C67 | The landlord bot ranks land by last year's rent as paid, so it chases wherever the harvest was best |
| R199 C64 | Nothing stops harvests running without the merchants' prices, where they feed the false swing |
| R202 C26 C95 | PLAN.md's stage 1 and README.md's Known problems still describe the Python port, and read as open what the game's rules answered |
| R106 C119 | The viewer was not looked at with kept town ground and does not read its three new layers |
| R37 C55 | The richer budget set is a judgement; new consumer goods need shares_rich written and the tables looked at again |
| L23 K10 | The buttons use Bevy's deprecated Interaction component |
| L46 K1 K35 | world::class_share is unused and tools/compare_classes.py is superseded but still in the README |
| L24 K22 | In a live game a works that outgrows its ground makes its cell lay out again and younger works move |
| L169 K25 | The painted hedge shadows assume a fixed sun, but the seasons move it |
| L93 K24 | A town whose middle is drowned ground has its market cross, paving and station in the water |
| L167 K19 | Carts and road ribbons follow the carved river bed under the bridge deck |
| L192 L166 | Traffic is in no layer of the record, so it cannot be mapped |
| L71 C118 | Some works are drawn smaller than their size for want of room, and some overlap neighbours |

---

# Part 4. The counts

| | Rules pages (R) | Look and interface (L) | Core reports (C) | The code (K) | All |
|---|---|---|---|---|---|
| open, as checked | 160 | 149 | 91 | 22 | 422 |
| in flight, as checked | 23 | 48 | 7 | 5 | 83 |
| fixed, as checked | 10 | 10 | 22 | 0 | 42 |
| moot, as checked | 8 | 11 | 10 | 2 | 31 |
| not a fault, as checked | 2 | 6 | 10 | 13 | 31 |
| **rows** | 203 | 224 | 140 | 42 | **609** |

After merging (R84 goes from open to moot with its twin C6):

- **Kept: 504 rows, 391 faults.** Open: 421 rows, 328 faults. Wholly in flight: 83 rows, 63 faults.
- **Pruned: 105 rows.** Fixed 42, moot 32, not a fault 31.
- **Duplicates merged:** 90 groups covering 203 rows.
- **The 421 open rows:** 296 in the 28 pieces of work (229 faults); 63 among the smaller things (54 faults); 25 for Stella's core loop (19 faults); 37 that are her decisions or matters of taste (26 faults).
- **The 28 pieces by rank:** 10 that break play or make the picture untrue (78 rows), 4 that other things wait on (54), 3 she has raised more than once (24), 5 that make the economics truer (85), 6 of polish (55). Ten of the 28 belong to an item already under 'Ready next'.
- **When their time comes:** at once, piece 23 (in part), four guards in the core and three single lines; everything else after a named branch lands or with a 'Ready next' item's trigger.
- **By kind, among the 421 open rows:** economics untrue 137, polish 99, test gap 64, picture untrue 49, taste for Stella 38, performance 19, docs 8, breaks play 7.
