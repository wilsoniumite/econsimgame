# The architecture, reviewed: what holds, what endangers the work, and the few changes worth making

12 November 2026. The question was: "how far are we from needing a big refactor of the codebase or an architecture review?" The answer given was that no big refactor is due yet and a review is. This is the review, written for whoever decides next week's work.

It rests on five surveys of the code made the same day (the rules as modules; the month and the year; the viewer; the game and its interface; the generators, tests and tools), each with files, lines and counts, and on the code itself wherever a claim decides something. The surveys read main at `fac6b42`. No file under `crates/` has changed between that commit and the one this page was written on, so their line numbers stand. Nothing was built or run. Where the surveys disagreed, the table at the foot says what a recount found.

`core/`, `viewer/` and `gen/` stand for the `src/` folders of the three crates: `core/econ.rs:283` is line 283 of `crates/isostate-core/src/econ.rs`.

## 1. The answer in short

The code is a long way from needing a large refactor, and none of the work planned for the coming weeks needs one first. The simulation's two spines (one `Params`, one `State`), the narrow channel between the player and the simulation, and the scenery built off the main thread from plain data are all sound, and the proofs that a change moved nothing are better than most projects have. What is due instead is small: nine changes of two to eight helper-hours each, four of which only add tests. Three matter most. The game's rules need a byte-for-byte test of their own, because today's golden test cannot see them. The viewer's ground needs a seam before a second maker is built beside it. And the list of a year's meshes, written out by hand in eleven places, should become one table before the next mesh is added. The one piece that resembles a refactor, lifting the 887-line monthly step into named stages, is right but not yet: five pieces of work already in hand edit that function, and it should wait for the pause the rules pass itself leaves. Even then it is about fourteen helper-hours of moving code that does not change, and not a rewrite.

## 2. The code as it is

| crate | size | what it is | what everything hangs on |
|---|---|---|---|
| `isostate-core` | 28,999 lines of source, 228 tests | The port of the Python model and the game's own rules, each behind a switch. | `Params` (`core/params.rs:100`: 87 numbers of the port, 15 for the families with `end_year`, and 16 rule structs holding 178 fields). `State` (`core/state.rs:155-534`: 214 fields, 70 of them for rules the Python never had). The month, `econ::step_month` (`core/econ.rs:283-1169`), and the year, `sim::yearly` (`core/sim.rs:410-496`). Outward, `Record` (`core/record.rs:141`) and `LiveYear` (`core/live.rs:248`). Inward, `save::Order` (`core/save.rs:73`) through `Game::give` (`core/save.rs:364`). |
| `isostate-viewer` | 31,116 lines in 36 files, 1,669 lines of shader, 174 tests | The Bevy viewer, which is also the game. | `Replay` (`viewer/replay.rs:145`), filled alike by a recorded run and a live game. `Ground` (`viewer/world.rs:85`), made once by `build_ground` (`viewer/world.rs:286`). The year's scenery, built off the main thread by `build_scenery` (`viewer/main.rs:1451`) and handed over by `viewer/handover.rs`. `live::Cmd` and `live::Msg` (`viewer/live.rs:35`, `93`). |
| `isostate-gen` | 12,968 lines in 41 files, 130 tests | Everything drawn, as plain functions with no Bevy. | `MeshBuf` (`gen/mesh.rs:11`), the kit of parts (`gen/kit.rs`, imported by 18 files), and a registry of 37 generators held to one scale by a test (`gen/registry.rs:345`). |

**The hot files.** Commits since 9 November that are not merges, of 633 in all:

| file | commits | why it is hot |
|---|---|---|
| `viewer/main.rs` (3,208 lines) | 69, which is 36 per cent of the viewer's 191 | The crate root, the command line, shared state and the year's rebuild in one file. |
| `viewer/scatter.rs`, `viewer/game.rs`, `viewer/live.rs`, `viewer/world.rs` | 37, 34, 28, 25 | Where the picture and the game are being built. |
| `crates/isostate-viewer/assets/look.toml` | 31 | Every art branch adds colours to one table. |
| `core/facts.rs`, `core/params.rs`, `core/econ.rs`, `core/lib.rs`, `core/state.rs` | 23, 21, 20, 20, 19 | The lists every rule appends to, and the month every rule hooks into. |

**How a change is proved safe.**

| proof | what it holds | what it cannot see |
|---|---|---|
| The golden test (`crates/isostate-core/tests/golden.rs:63`): seed 7 to 1900 under `default_params()`, with no families and with six, the monthly series and the news hashed. | The port, byte for byte. | Anything that happens only under the game's rules. |
| The sweep: eight islands under the game's rules with six families, set against a reference, seed for seed; "0 facts moved". Fact names are frozen by a test (`crates/isostate-core/tests/sweep.rs:290`). | The game's rules, as some 450 summaries an island. | A change that alters the run without moving a summary. |
| The twelve standard pictures at 0 pixels (`viewer/shots.rs:148-204`, compared by `compare` at `:438`). | The old island as drawn. | The panels (every picture is taken with the side panel shut, `viewer/shots.rs:140`), and the new land. It needs a graphics adapter and minutes. |
| `ISOSTATE_MESH_SUMS` (`viewer/main.rs:1463`): a fingerprint of every year mesh. | The scenery, byte for byte, without a graphics adapter. | The ground's shader. |
| 532 tests, 10 of them ignored. | Each rule, the saves, the shader against the Rust. | See findings 3, 4 and 8 below. |

## 3. In good shape: leave it alone

So that nobody tidies it.

- **The convention for a rule** (`core/params.rs:15-65`): its own module, its own struct, `Default` off, `game()` on, and left out of the port's `params.json` while off. All 16 structs follow it. The sweep's `--set` and the file loader need no table of names (`core/sweep.rs:313`).
- **Randomness.** The month draws no random number; every draw is in the year, one stream to a rule. The year (`core/sim.rs:410-496`) is already an ordered list of 28 calls in 87 lines.
- **The labour market** (`core/econ.rs:188-252`, `421-567`): exact, and tested against the plain bisection it replaces.
- **The player's seam.** Six kinds of `Order` (`core/save.rs:73-84`), one entry (`Game::give`), one dispatcher (`core/save.rs:140-164`) into six functions of `core/families.rs`. The simulation runs on its own thread; the viewer can send it 11 commands and read 6 answers (`viewer/live.rs:35-104`), and the scenery cannot see that channel at all. This is the seam to keep narrow, and it is narrow. It is wide only outward, by design: `LiveYear` has 25 fields (`core/live.rs:248-300`). Its one unguarded place is `Order::for_year_end` (`core/save.rs:114-116`), a `matches!` with a default, so a new kind of order that should act at once would be treated as word to the family without the compiler saying so.
- **Saves as a log of orders** (`core/save.rs:1-33`), written whole or not at all (`viewer/saves.rs:1-26`).
- **The core works out what the panels show** (`site_options`, `core/families.rs:1071`; `family_report`, `:1513`), so the interface rarely repeats economics.
- **The off-thread build and the budgeted hand-over** (`viewer/handover.rs`, 8 tests), and the one ordered chain of systems held by a test (`viewer/main.rs:1071`, `2729`).
- **The shader's twin** (`viewer/shader_twin.rs`), which runs the shader's own source against the Rust bit for bit.
- **The generator crate, whole**, and the houses within it above all.
- **`core/families.rs` (2,158 lines) and `core/accounts.rs` (1,918)**: large, each about one thing, already in named sections.
- **The proofs themselves**: the golden test, the sweep's comparison, `live_tests.rs`, the script runner.

## 4. What slows or endangers the work

In order of danger. "Cost so far" is what the history shows, and where it shows nothing the table says so.

| | finding | evidence | cost so far |
|---|---|---|---|
| 1 | **The game's rules have no byte-for-byte guard.** The golden test runs only `default_params()`, under which most of the month's hooks return on their first line. The month has contracts no test states: `st.wage` is last month's until `core/econ.rs:1104` and three rules divide by it before then (`core/carting.rs:74`, `core/canals.rs:477`, `core/roads.rs:664`); `st.fill` is last month's until `:1103`; four rules take hands from the labour supply in an order that is part of the result (`:424-432`). | `crates/isostate-core/tests/golden.rs:63-64`. No other file in the three crates stores a hash of a run. | Nothing traced to it yet. Two merges came close: in `506f368` a test of the people rule failed when the navvies joined the labour supply, and in `de0a6d3` the place of the towns' hook in the year had to be reasoned out in the merge message. A hook misplaced by a merge would pass the golden test and be caught, if at all, by a fact. |
| 2 | **Lists kept by hand in several places, with nothing to check that they agree.** A new rule touches 7 to 11 shared files (roads 7, mines 9, canals 10, accounts 11). The kinds of news are bare strings known to three tables. A year's mesh is written out in eleven places (`viewer/main.rs:1374-1658`, `viewer/handover.rs:255`). | The list of mechanisms at `docs/rules/README.md:462-522` lacks commons and roads. `README.md:110-127` says twelve mechanisms and never names commons, which `game_params()` switches on (`core/params.rs:549`). `core/params.rs:306` sends commons' readers to the mines' page. "emigration" (`core/peopling.rs:218`, on in the game) is in none of `core/chronicle.rs:1202`, `viewer/ui.rs:1063` or `:1082`. `README.md:100-101` says 210 and 127 tests where there are 228 and 130. | Nineteen merge messages since 9 November mention conflicts, nearly all two branches appending at the same place: five in `7c9fac1`, five in `de0a6d3`, four in `2782f33`, three in `506f368`. And one fault is on main today: people sailing from the island are filed as grey "News" (`viewer/ui.rs:1078`, `1103`). |
| 3 | **Tests that time the machine.** Eleven assertions bound wall-clock time or wait a fixed while, on a machine shared by two dozen builds. | A median save under 100 ms (`viewer/saves.rs:777`); `live_year` under 100 ms (`crates/isostate-core/tests/families.rs:306`); refusals within 1 s (`viewer/saves.rs:921`, `viewer/live_tests.rs:641`); polls of 5 s (`viewer/live_tests.rs:432`, `656`). | The first of these failed on the day of the surveys, under load. A false failure stops a merge and teaches helpers to run again until green. |
| 4 | **The houses are bound by a promise and guarded only by pictures.** They are built from parts outside their folder, and their `roof.rs` decides the roofs of six other kinds of building. | `gen/kit.rs` is imported by 18 files; `gen/houses/roof.rs` by `gen/works/kit.rs:27` and five more. The fingerprint printer is ignored and compares nothing (`gen/sheets.rs:810-835`). | None recorded. A works branch that moved a house would be found only by pictures, minutes later, on a graphics adapter. |
| 5 | **The ground has one maker, and much is asked around it.** | `build_ground_inner` (`viewer/world.rs:290-335`) is the only maker. 33 lines in 14 files read the run's `terrain` or `elev` directly to ask whether a place is land or upland. One global cache is not keyed by island (`viewer/scatter/towns.rs:241`); another holds a single year for three askers (`viewer/rails.rs:1034`). | None yet. The land-view critic's findings (a fifth of the first towns with their market square on tidewater or in a lake; lakes counted as dry ground when a district's land is shared out) are this finding seen from the other side. |
| 6 | **The month is one function that every rule edits.** | `core/econ.rs:283-1169`: 887 lines, 245 `let` bindings, 45 calls into 14 modules, 49 locals handed to the books (`core/accounts.rs:574-633`) and 28 still alive in the closing bookkeeping. Rules keep themselves off in three ways (their own first line, the caller's `if`, or empty data). | One recorded conflict, at the labour supply (`7c9fac1`). It slows more than it endangers once finding 1 is mended. |
| 7 | **`viewer/main.rs` is root, hub and hot spot at once.** | 3,208 lines; 15 modules use 48 of its items; about 390 lines are the command line, 334 are `fn main`, 370 the year's rebuild. | One recorded conflict (the module list, `c11f15a`), and 3,208 lines for a helper to read on 36 per cent of the viewer's commits. |
| 8 | **Clicks rest on a deprecated component and on tests that are not run.** | `Interaction` in 113 lines of 7 files, read by 22 systems; the engine's own note says it goes after 0.20. The only four tests that turn a click into an order are ignored (`crates/isostate-viewer/tests/saves_guard.rs:390`, `419`, `488`, `688`). | None yet. A broken button would reach main unseen. |
| 9 | **A save cannot say which ground it was made on.** Orders carry cell numbers (`core/save.rs:76-83`), one unreadable order refuses the whole file (`:479`), and the rules' fingerprint leaves out 14 of the families' numbers (`core/params.rs:208-248`). | As cited. | Every rule merged already makes older saves drift; that larger matter is the backlog's "Ready next" 9. |

## 5. The four guesses

| guess | verdict | the number that decides it |
|---|---|---|
| 1. Each rule should own its settings, facts and hooks in one place. | **Wrong as a refactor; right as a rule for new code.** | All 16 settings structs already live in their rule's module, and a new rule adds 5 or 6 lines to `core/params.rs`. The collisions are real (19 merges) but each was two appended blocks, both kept. The hooks cannot move: 45 calls stand inside the month's arithmetic, and six rules are replacements in an `if` of the port's own code (`core/econ.rs:156`, `308`, `1008`; `core/rail.rs:42`; `core/sim.rs:38`, `436`). What is missing is not ownership but a check that the lists agree. |
| 2. The monthly step should be an ordered list of named stages. | **Right later, and in a plainer form.** | It is provable: the month has 0 random draws and 0 early returns, so a lift that keeps every expression keeps every bit. But 5 pieces of work in hand edit it: the production tree's waiting branch changes 45 lines in the demand, requests, execution, inventories and trade stages, and the rules pass's stages 2, 4, 6 and 9 each add to it. The form is named functions that pass typed structs, not a list behind one interface: 49 locals go to the books. |
| 3. The viewer's scenery should be layers behind one interface. | **Half right, and that half now.** | 11 places to edit for one new year mesh, and three off-thread makers that already take the same inputs. But only about 550 of `main.rs`'s 3,208 lines are scenery, and the makers on the main thread (boats, carts, flags) are already plugins. |
| 4. The interface should come off the deprecated component. | **Right later.** | 113 lines, 22 systems, 5 presses written by hand in the test tools, and 2 engine crates this build has never compiled. The next engine upgrade forces it; nothing on the coming list needs it; the tree's panels are open in the same files. |

## 6. The changes worth making

Each lands alone and is small enough for one helper. "Design" means the first pass wants the stronger model; "mechanical" means Opus. If only three are made, make A1, B1 and B2.

### Before the rules pass begins

All four add tests or mend lists. None changes a line that runs in a simulation.

| | what | files | proof that it moved nothing | hours, kind | risk |
|---|---|---|---|---|---|
| A1 | **A golden test under the game's rules**: seed 7, `game_params()`, six families, to 1900, the same two texts hashed and the same way of retaking the constants. With it, a comment at the head of `step_month` stating the contracts of finding 1. | A new `crates/isostate-core/tests/golden_game.rs`; a comment at `core/econ.rs:283`. | It adds a test and a comment. The port's golden test passes by construction; the sweep shows 0 facts. | 2, mechanical | Every stage that changes a rule on purpose must retake the constants on the merged tree, and two such branches will conflict on those lines. That is the right place for them to meet. It adds one run of 15 to 20 seconds to the core's tests. |
| A2 | **One list of the rules and one of the kinds of news, and a test that the hand-kept lists agree with them.** Then mend what it finds: commons and roads in the two READMEs, the pointer at `core/params.rs:306`, labels for "emigration" and "turnpike", and `Params::check` called on swept settings. | `core/params.rs`, `core/chronicle.rs:1202`, `viewer/ui.rs:1063-1105`, `core/sweep.rs:354-356`, a new test file, the two READMEs. | Golden test; sweep at 0 facts; pictures at 0 pixels, since no standard picture shows the news. One change is meant to show: an emigration gets its own label. | 3, mechanical | Low. A renamed page now fails a test, which is intended. So will the production tree's branch on its trial merge, until it adds its own line to the list. |
| A3 | **Speed and correctness in separate tests, and the gate written down.** The two speed claims go behind `#[ignore]` with a line saying when to run them; the waits that are not speed claims are widened to 30 to 60 seconds. A page states the gate as it is run, since today no tracked file does, and `README.md:100-101` is corrected. | `viewer/saves.rs:777`, `921`; `viewer/live_tests.rs:432`, `641`, `656`; `crates/isostate-core/tests/families.rs:306`; a new `docs/CHECKS.md`. | Test code and documents only. | 2, mechanical | None, provided the speed tests are named on the page, or they are lost. |
| A4 | **The houses held by a test.** The ignored printer becomes a test of the house sheets against stored constants, taken on main first. | `gen/sheets.rs:810-835`. | It adds a test. | 2, mechanical | Trigonometry can differ in the last bit between machines; hash positions rounded to a tenth of a millimetre. |

*Why now.* The pass's stage 1 (the card) and stage 5 (sixteen settings made constants) each claim that 0 facts differ; A1 makes that 0 bytes, and every later change to the core leans on it. The pass adds three mechanisms (hands, money, invention) and ends by rewriting every rule's page, so A2 is exercised at once. The pass is read on sixteen islands with null pairs, on the machine the gate runs on, which is when A3's false failures cost most. And the works' site planner and the tree's new works are editing the kit the houses are built from.

*What they unblock.* A1 unblocks C1 and every later change to the core that claims to move nothing. A2 lets a new rule, a new kind of news or a new page be added without a list being forgotten. A3 lets the gate be believed when it fails. A4 lets the kit, the roofs and the colours be edited freely, because a moved house now fails a test in seconds.

### Beside the new island

| | what | files | proof that it moved nothing | hours, kind | risk |
|---|---|---|---|---|---|
| B1 | **The ground's seam.** Split `build_ground_inner`, unchanged, into the landform (`:291-317`) and the finishing of a ground from heights (rivers `:319-322`, field system `:323-334`), so that the new maker replaces only the first. Give `Ground` the questions "is this land?" and "is this upland?", and move the 33 direct reads to them. Key the towns' cache by island. | `viewer/world.rs:290-335`; the 14 files of finding 5; `viewer/scatter/towns.rs:241`. | A test that hashes every field of `Ground` bit for bit on three seeds, taken first; `ISOSTATE_MESH_SUMS` the same on eight islands; pictures at 0 pixels. | 6; design for what the two questions mean at a lake or a tideway, then mechanical | Low to middling. `scatter/towns.rs` and `kept.rs` are open in the towns' second pass, so their nine reads follow its landing. |
| B2 | **One table of a year's meshes, and the year out of `main.rs`.** First move `viewer/main.rs:1371-1741` unchanged into a file of its own. Then one table (name, material, whether a camera move resends it, place in the hand-over) from which `Handles::year` (`:499`), `year_parts` (`:1392`), the index constants (`:1410-1416`), the drop list (`:1572`), `hand_over_order` (`:1654`) and the names at `viewer/handover.rs:255` are all derived. | `viewer/main.rs`, `viewer/handover.rs`. | `ISOSTATE_MESH_SUMS`; the script's `scenery` check (`viewer/main.rs:1714`); a test that the derived hand-over order is today's (`:1657`); pictures at 0 pixels. | 8, design | Middling. The hand-over's order was hard won (`viewer/main.rs:1648-1653` says why); its 8 tests hold it. |
| B3 | **A save names its ground, and one unreadable order no longer refuses the file.** The ground's name defaults to the old island, so every existing save reads as before; an order this build cannot parse is refused and counted, as an order the simulation refuses already is. | `core/save.rs:192-213`, `:479`, `:548-571`. | The core's save tests, `viewer/live_tests.rs`, and the test that a save in the first layout still loads (`crates/isostate-viewer/tests/saves_guard.rs:550`). | 3, mechanical | Low. It changes the save's file, not the orders, so the player's seam is untouched. |

*When.* B1 must land before the island's stage 1, whose first line is that `build_ground_inner` "chooses between today's maker and a new `ground_from_land`" (`docs/probes/dx-land-view.md`, section 6); stage 0 touches other files and can go beside it. B1 is also where the critic's lake mask is answered once and not in fourteen files. B2 must land before the first new year mesh: the canals and workings waiting to be drawn (the backlog's "Ready next" 2), then the island's roads. B3 must land before the switch is thrown.

*What they unblock.* B1 unblocks the island's stage 1 and gives every later stage one place to say what is water. B2 makes each new thing drawn by the year (a canal, a working, a road net on the new rivers, later the tree's works) one row and one maker. B3 lets a game started on the old island be told from one started on the new.

### After both

| | what | files | proof that it moved nothing | hours, kind | risk |
|---|---|---|---|---|---|
| C1 | **The month lifted into named stages**, one stage to a commit and no line reordered: trade (`:874-969`), prices (`:992-1020`) and demand (`:295-341`) first; then the rest; the labour market (`:421-567`) whole, never split, with the four rules that take hands as its opening lines. Each stage returns a small struct, and the books' 49-field `Month` shrinks to references to them. In the same piece, the seven hooks whose switch the caller tests today (at `:308`, `395`, `774`, `788`, `1008` and `1040`) come to test their own. | `core/econ.rs`, `core/accounts.rs:574-633`. | Both golden tests after every commit; the sweep at 0 facts; pictures at 0 pixels. | 4 for the first three stages, about 10 for the rest; design | The highest here: a write to the state moved across a read. A1 exists to catch exactly that. |
| C2 | **Clicks behind two helpers**: one that reads a press for the 22 systems, one that makes a press for the test tools (`viewer/script.rs:356`, `viewer/frames.rs:1295`, `viewer/game.rs:1649`); the six module-wide `allow(deprecated)` made item by item. The move to the engine's picking is then a change to `viewer/ui.rs` and the places buttons are spawned. | The 7 files of finding 8. | A fixed script's printed text the same before and after; the four ignored scripted tests run by hand on a graphics adapter; no core file changes. | 4, mechanical | Low. |

*When.* C1's earliest moment is the pause the pass itself names, the tree's landing between its stages 5 and 6 (`docs/designs/rules-pass.md`, section 5), and only if no branch is then open in `core/econ.rs`. Before that it would collide with the tree's branch in two of the first three stages to lift. If the pause is missed, it waits for stage 10. C2 waits for the tree's panels to land and should be done before any engine upgrade is begun.

*What they unblock.* After C1 the pass's later stages (the freight, the land's hands) and every rule after them edit a named stage of thirty to a hundred and fifty lines and not one function of 887. After C2 an engine upgrade no longer has to rewrite the test tools' hand. Nothing in this list is for the run to 2050 alone: what that needs first is a measurement, and section 8 names it.

### Never

| what looks tempting | why not |
|---|---|
| A trait for rules, a registry of boxed rules, or a macro that writes `Params`. | It would lose the plain reads of a rule's settings and hide the attribute that keeps the port's `params.json` byte for byte, to save 5 or 6 appended lines a rule. |
| The month as a list of boxed stages behind one interface. | The stages hand typed data to each other; one interface would push it all back into `State`. |
| Any change inside the labour market. | It is exact and tested, and the most delicate arithmetic in the crate. |
| Gathering the 44 loose fields of `State` into sub-structs as a campaign (the mines have 19, `core/state.rs:475-519`). | A rename across files that four branches have open, which endangers nothing today. Do it for one rule when that rule is opened for another reason, as the pass opens the mines. |
| Splitting `families.rs` or `accounts.rs` along their sections. | Each is about one thing; `families.rs` is where the core loop lives and should be left for its designer. |
| One spelling for a decision where the viewer has three (`Action`, `Cmd`, `Order`). | It reaches into the player's seam. Six files for a new kind of decision is the price of keeping the simulation on its own thread; change it only at the designer's word. |
| Untangling the viewer's import cycles, or putting boats, carts and flags behind B2's table. | One crate, so the cycles cost nothing; the main-thread makers are already plugins with their own guards. |
| A port crate and a game crate. | Six rules replace port rules inside the port's own branches. |
| Moving `main.rs`'s command line to its own file for its own sake. | A move does not lessen the four places a new option edits. It is harmless if a quiet hour offers, and not worth scheduling. |
| Leaving the deprecated component before the engine forces it; replacing the 36 environment variables with a configuration system; touching `sim::yearly`. | None of the coming work needs any of them. |

## 7. Rules for new code

1. A new rule keeps its state in one sub-struct of `State` with its own `Default` and its own random stream, never in loose fields (finding 6; the mines' 19).
2. A hook tests its own switch on its first line, and the comment at its call says what it reads from last month and why it stands where it does (findings 1 and 6).
3. Whatever is appended to a hand-kept list (a rule, a kind of news, a year mesh) is added to that list's one table, and the table's test is run (finding 2).
4. A change that claims to move nothing does not alter a stored hash, and a change that means to move something retakes the hashes on the merged tree and says so in its commit (findings 1 and 4).
5. Ask the owner: the interface imports or is sent a rule's number, and the scenery asks the `Ground` whether a place is land or water, and neither types it again (finding 5; "a fifth" at `viewer/game.rs:850`, age 18 at `viewer/family.rs:183`).
6. A test asserts correctness or speed, never both, and a test of speed is ignored by default (finding 3).

## 8. What would change this verdict

A larger refactor has become due when any of these is seen.

- **A hook misplaced.** The game's golden test fails after a merge and the cause is the order of two hooks, or a merge in the month needs an order decided and not merely both sides kept, a second time. Then C1 is done at once, whatever is open.
- **A third rule inside the labour market.** The pass's stage 2 acts after the wage is found and its stage 9 changes one number within. A third rule that must act inside the two closures means that stage needs a designed interface, and a review of its own.
- **Two grounds leaking.** After the island's stage 1, more than about five places outside `viewer/world.rs` that ask which maker made the ground. Then the ground wants an interface and not a seam.
- **The table not holding.** A new year mesh that still costs more than two edits after B2.
- **`State` passing 300 fields**, or a second rule reading another rule's loose fields as the mines read the land's.
- **The run to 2050.** The works' table has 20,000 slots that are never reused (`core/params.rs:387`, `core/firms.rs:186-190`) and nobody has counted how many are used by 1900; count it before that work begins. The town ground's year is a byte from 1645 that is full at 1900 (`core/record.rs:31`), and loading a save replays every year. If the slots pass half, or the record's bytes must widen, the record and the save together become the first truly large piece, and it should be designed as one.
- **A second set of game rules** (an era for 2050 with its own values), which the flat list in `game_params()` was not made for.
- **The gate failing its purpose**: more than half an hour a merge, or a false failure more often than one run in ten after A3.
- **An engine upgrade put on the calendar.** Then guess 4 is due, with C2 first.

## What was checked, and what a recount found

| claim | found |
|---|---|
| The month's length: one survey gave 890 lines to line 1172, another 887 to 1169. | 887. The function closes at `core/econ.rs:1169`; the longer count ran into the next function's comment. |
| Fields of `State`: 214 or 212. | 214: 144 of the port, counting the observer's two, and 70 of the game's rules. |
| Settings structs in their rule's own module: "14 of 16", with the works' and the towns' in `params.rs`. | 16 of 16 (`core/works.rs:49`, `core/towns.rs:51`). |
| Calls from the month into the rules: 43 by one count, 40 by the other. | 45 calls into 14 modules, 13 of them into the families, and 4 direct reads of a rule's settings. |
| Commits to `viewer/main.rs` since 9 November: 90, 88 or 69. | 69 that are not merges, of the viewer's 191, which is the 36 per cent; 90 with merges. |
| Merge messages that mention conflicts: 20. | 19 by a search for the word. |
| The first three stages to lift are trade, prices and demand. | Right for a quiet tree. The production tree's waiting branch edits two of the three (`core/econ.rs:294` and `:875-941` in its diff against main), which is why C1 waits. |
| An emigration is shown as grey "News". | Confirmed on main (`viewer/ui.rs:1078`, `1103`). |
| Not checked: that the timed save test failed on the day (taken from the survey); how many works' slots are in use by 1900 (it needs a run); what the engine offers in place of the deprecated component (those crates have never been downloaded here). | |
