# dx-save-state: a saved game that holds the island as it stands

Branch `night/o-dx-save-state`, cut from `main` at a6ff0ec on 12 November 2026. It adds one file, `crates/isostate-core/examples/save_state.rs`, and changes nothing under `src`. The example weighs every part of the island's state, follows which parts change and when, and writes a rough picture of the whole run to bytes and reads it back, to see whether a game gone on with from the picture is the same game to the last digit. It is backlog piece 10 of `docs/backlog/found-along-the-way.md` (R139, L12, L13, K16, R30), measured and designed, not built.

Islands 7 and 3, six families, the game's rules, played a month at a time as a game is played, with no orders. **The machine was very busy** while this was measured: some ninety builds and runs of other helpers at once. A release build of the core took 9 m 44 s, and `isostate-core run --seed 7 --rules game --families 6`, which the README gives as 15 to 20 s, took 2 m 58 s. Every time below was taken under that load, and is best read against the century played under the same load, which is given beside it. Sizes do not depend on the load.

## In short

1. The state of a game is about 56 MB in memory in 1750 and 96 MB at the end of 1900 (island 7; island 3 is the same in 1750 and 2.4 per cent smaller in 1900). Most of it is not the economy. At the end of 1900 the books of account are 44 MB, the yearly record the viewer's history is drawn from 22 MB, and the transport tables 8 MB; everything else is some 21 MB, and 5.4 MB of that is slots for twenty thousand works of which 2,320 have been used.
2. Of 217 parts (the state's 213 fields and the run's streams, news, record and clock), 62 change every month (about 7 MB in 1900, and the books), 94 at every year's end (about 40 MB, most of it the record and the transport), 32 now and then, and 29 never after the start (the island itself, the 1750 baselines, a few switches).
3. Written raw with nothing but the standard library, the run at the end of 1900 is 51 MB without the books and 95 MB with them; `gzip -6` takes the 51 to 10.8 MB and `xz -6` to 8.0. Writing it took 0.06 s and reading it 0.09 s, while playing the century again took 111 s under the same load: a load from a picture would be a thousand times quicker than a load from the log of orders.
4. Read back and played on ten years, from the turn into 1850 and from six months into 1850, the game is the same to the last digit on both islands: the save's own check, every byte of the state and the record, the families and the books (the last two carried across in memory, not written, as an example cannot reach their private fields). So a picture of the state is a sound way to save, and nothing in the state needs to be worked out again on loading.
5. Three things make it harder than it looks. The random streams cannot be read from outside (`rand_pcg` hides its state; its `serde` feature, already in the lock, would show it). The families and the books keep private fields, so the writer has to live in the core. And the transport tables cannot be worked out again on loading: the graph reads the woods, which the year's land use moves after the graph is made, so tables made again on loading differ at all sixteen points sampled, and a game gone on with from them is a different game.
6. The design keeps both kinds of save. The log of orders stays the save of record, for honesty, replays and a change of rules; a picture of the state is written beside it at year's ends and read when it was made by the same build. A picture made by older code is used only where its parts can all be read, and the card then offers to go on from where the player left it under the new rules, or to play it again from 1750.
7. Build first: the picture, written beside the log and read only by a test that holds it to the log, in the core (one module and a few lines in `accounts.rs`; the families' hidden fields are already `pub(crate)`), with the streams written by their state. Then the load from it, which needs the viewer to draw its history from the record and the record to keep owners and the families' worth by year.

## 1. What the state is

`State` (`state.rs`) is one struct of 213 fields. Hanging from it are the island (`World`), the cost graph (`Graph`), the month's figures (`MonthStats`), the series and the observer's memory, the families (with their members and decisions), Engel's, the chronicle's and the machines' memory, the railways' book (`WayBook`), the canals' state (with a book of its own and a random stream), the people rules' founding stream, and the books of account (`Books`, with its history behind an `Arc`). Outside the state, the run (`sim::Sim`) holds the content and the parameters (both made from the rules), eight random streams, the news, the yearly record (`Record`, from which `run.bin` is made), and the clock (`year`, `end_year`, `t`). A game (`save::Game`) adds the log of orders, those waiting, the months played of the open year, when it began, and the outcomes of the last year's end until they are handed over.

Every map in the state is a `BTreeMap` or `BTreeSet`, which is ordered, except one: the series' index by name, a `HashMap` that only finds a series' place and is made again from the names.

Size in memory (what a deep copy allocates), island 7, from `--seeds 7`:

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| everything | 55.6 MB | 66.7 MB | 80.3 MB | 95.8 MB |
| the books of account | 12.3 | 20.4 | 31.1 | 44.1 |
| the record (`rec`) | 18.1 | 19.1 | 20.4 | 22.1 |
| the transport (`graph`, `dist`, `pred`, `settle`, `catch`, `dcatch`, `town_cost`, `delta`) | 6.5 | 7.5 | 8.2 | 8.2 |
| the works' slots (25 parts) | 5.3 | 5.4 | 5.6 | 5.8 |
| the mines' tables | 4.2 | 4.2 | 4.3 | 4.3 |
| the series | 1.5 | 1.5 | 1.5 | 1.5 |
| the chronicle's memory | 0.0 | 0.4 | 0.8 | 1.2 |
| the island itself (`world`) | 1.0 | 1.0 | 1.0 | 1.0 |
| everything else | 6.7 | 7.2 | 7.4 | 7.6 |

Island 3: 55.6, 66.6, 79.2 and 93.5 MB. The content and the parameters are 21 kB together and are not written: they come from the rules.

Some of what is held is room for later. In 1750 the record has all 151 years made ready and empty (18.1 MB), the series all 1,812 months (1.5 MB), and the works 20,000 slots (5.6 MB, of which 258 are used). Trimmed to what has been used, the picture would be about 30 MB in 1750, 47 in 1800, 67 in 1850 and 90 in 1900. The books carry about 12 MB from the first month, the open year's tables sized for twenty thousand works, as the books' checker found.

The books grow by about 0.2 MB a year and the record by 0.03 MB; the rest of the state hardly grows. A run to 2050 would have a record of about 40 MB, books of some 75 MB and series of 3 MB: a picture of 2050 raw would be about 160 MB with the books.

The random streams at the end of 1900, numbers drawn (world, names, firms, tech, people, rail, misc, harvest): island 7 40,147, 12, 1,254,500, 526, 0, 0, 671,285 and 7,904; island 3 the same but for firms 1,100,280, tech 621 and misc 611,865. The people and rail streams are the port's and are not drawn from under the game's rules; the game's own founding and canal streams sit in the state.

## 2. What changes and when

Each part was fingerprinted after every month of eight sample years (1751, 1752, 1800, 1801, 1850, 1851, 1899, 1900: 88 ordinary months and 8 that close a year) and at every one of the 151 turns. On island 7:

| class | parts | weight in 1900 | examples |
|---|---|---|---|
| every month | 62 | about 7 MB, and the books' 44 | prices, stocks, wages, cash, the works' capacity, cash, orders and use, the series, the woods' cover, the mines' running sums, the families |
| some months | 7 | small | `rail` and `rail_opened` (a line opens), `projects`, the railway and canal books, `food_min_acc`, `mine_crew_each` |
| every year's end | 94 | about 40 MB | the record, the news, the transport, land shares and bids, the works' list, the mines' tables, the chronicle |
| some years' ends | 25 | small | towns founded, inventions, the towns' kept ground, workings opened and left, the canal layer |
| never | 29 | about 1 MB | the island, `water`, the 1750 baselines (`disc_X0`, `cpi_basket`, `money_start` and the rest), the tech indices, the switches and tables kept for tests and probes, `hh_income` and `lord_income` (never written under these rules), the port's railway book |

Island 3 gives the same classes but for two parts: `know` changes at some years' ends rather than every one, and `pp_emigrants` never rather than at three (93 at every year's end, 25 at some, 30 never). Only the 69 parts of the first two rows changed in any ordinary month; the other 148 stood still from one turn to the next. So a picture made in June differs from the turn's in some 7 MB of the state and in the books. The random streams are among those that change only at a year's end: under the game's rules every draw is made in the yearly step. And the history (the record, the news, the books' past years, the series) only ever grows: it can be written as an addition, year by year, and never written again.

## 3. Writing it down with no new dependency

**In `Cargo.lock` already.** serde 1.0.229 and serde_json 1.0.154, which the core uses; toml; ron 0.12.3 and bytemuck 1.25.2, which Bevy brings; rand_pcg 0.10.2, whose `serde` feature is off; and for packing, flate2 1.1.11 with miniz_oxide 0.8.9 and 0.9.1 (brought by png) and ruzstd 0.9.1 (brought by bevy_image, and it has an encoder), with crc32fast and adler2. No binary format for serde (no bincode, postcard, rmp or ciborium). None of the packers is a dependency of the core or the viewer: using one adds a line to a `Cargo.toml` and no new crate to the lock (only a line to the core's list of dependencies in it), which is a decision for the designer and not taken here.

**JSON will not do for the state.** It has no infinity and no NaN (`dist` is infinite wherever a town cannot reach and the series are NaN for months not yet played), the state is mostly long arrays of numbers that JSON writes at two or three times their size, and most of the structs derive `Serialize` and not `Deserialize`.

**What has to be written by hand, as the prototype shows.** A trait with a writer and a reader for numbers, strings, lists, options, fixed arrays, pairs and triples, ordered maps and sets (about 150 lines), and a line per struct listing its fields. The prototype's reader for a struct is a struct literal, so a field left out of the list does not compile: with 211 fields in `State` and some 25 structs under it, that is the guard that keeps the picture whole as fields are added. The series' index is made again from the names. The random streams need their state, and there are two ways: turn on `rand_pcg`'s `serde` feature (no new crate) and write the generator's two 128-bit numbers, or keep in `Rng` a count of the 64-bit words drawn and jump ahead on loading with `Pcg64::advance`. The prototype, which cannot see inside a stream, finds the count by drawing from a fresh stream of the same seed until three numbers agree (4 ms for 1.25 million draws) and reads back by drawing that many again. The families' five hidden fields and `Member::lifespan` are `pub(crate)`, and eleven fields of the books (with the whole of the open year's `Acc`, some twenty more) are private to `accounts.rs`; none can be reached from an example, so in the prototype they were carried across by a copy in memory and only weighed. A real writer goes in the core: there it reaches the families' fields as they stand, with no change to `families.rs`, while the books need their writer beside them in `accounts.rs` or their fields and `Acc` made `pub(crate)`. Under the game's rules the new land's island is never there (the land switch is off); with it on, `Landform` needs its own writer or to be made again from the seed.

**What it weighs.** Written raw by the prototype, the run (the state, the streams, the news, the record and the clock, without the books and the families) is 48.6 MB at the turn into 1850 and 51.0 MB at the end of 1900 on island 7 (50.9 on island 3). The books would add their 44 MB in 1900 and the families 30 kB. Packed from the shell (sizes in millions of bytes):

| island 7, end of 1900, without the books | raw | gzip -1 | gzip -6 | xz -6 |
|---|---|---|---|---|
| size | 53.5 MB | 12.1 MB | 10.8 MB | 8.0 MB |
| time to pack, loaded machine | | 1.4 s | 2.4 s | 17.6 s |
| time to unpack | | | 0.38 s | 0.53 s |

The turn into 1850 packs to 8.8 MB with `gzip -6`, and island 3's 1900 to 10.6 MB. Trimming the works' empty slots and the record's empty years before writing takes off what packs best anyway, so it matters for the raw file and little for the packed one.

**How long.** Under the load described at the top: the end of 1900 written in 0.060 s and read in 0.087 s; the turn into 1850 written in 0.11 to 0.42 s and read in 0.12 to 0.48 s as the load came and went. The century played a month at a time under the same load took 111 s on island 7 and 190 s on island 3, after a start of 4.7 and 12.8 s. On a quiet machine the century is 15 to 20 s by the README and loading a save of 1900 22.9 s (piece 10), so a raw read should be some tens of milliseconds. A packed read adds the unpacking, a few tenths of a second for gzip.

## 4. What must come back exactly

Today the save's tests (`tests/save.rs`, "thirty years saved and loaded are the same to the last digit", and the viewer's live tests) hold a game played again from its log to the game that was saved, by `save::check`: every month of every series, every family's cash and seat, and who owns each cell and each works. A picture has to meet the same test, and the prototype does, and more strictly:

- At the turn into 1850 and six months into 1850, on islands 7 and 3: written, read back and printed again, the bytes are the same; played on ten years beside the run it was taken from, `save::check` is the same, the state, the news and the record are the same byte for byte, and the families and the books print the same (once the books' `seconds`, the time they took on this machine, is left aside). The families and the books were not written: they were carried across by a copy in memory, since an example cannot reach their private fields. So the round trip shows that the rest of the picture is whole, and not yet that the families and the books can be written; that is for the writer in the core (section 8).

What makes it hard, and what each needs:

- **Floating sums.** Nothing, if every number is written as its bits. The danger is a text format or a rounding on the way (`serde_json` without `float_roundtrip` is one ulp off now and then); and any part worked out again on loading, which sums in another order or from other inputs.
- **The order of maps.** Every map in the state is ordered by key, so it comes back in the order it went. The one `HashMap` is an index, made again from the names. A `HashMap` added to the state later, iterated by the simulation, would break a game played on from a picture and a game played twice alike, which `save::check` would show.
- **Parts worked out again on loading.** Only safe where the part is a function of the state as it stands. The island is: made again from the seed in 0.07 s (island 7) and 0.25 s (island 3), it is the same to the last bit. The transport tables are not: the graph is made from the roads, the rails, the canals, the ports and the woods' cover, once a year, and the land is shared out afterwards, which moves the woods, while a railway that opens in May changes the rails and not the graph until the year's end. Made again from the cells as they stand, the graph, the paths and the market areas differed at all 16 points sampled on each island, at turns and in June, and made again at the turn into 1850 and played on ten years they gave a different game. They are 8 MB and must be written.
- **The random streams**, by their state or their count, as above.
- **The families' and the books' private fields**, written by code in the core.
- **The news and the viewer's history.** The live view sends the news by its place in the run's list (`LiveCursor`), and the chronicle marks earlier items by that place. A picture that carries the whole list keeps every place, so the stable name for each item that R30 asked for is not needed for this; it would be for a picture without the news. The viewer, though, builds its picture of past years (`Board` in `src/live.rs`) from `live_year` as the years are played again on loading. A game loaded from a picture plays no years, so the viewer would open with no past: no charts, no map of earlier years. The record has most of what it needs (it is what a replay of `run.bin` is drawn from), but not who owned each cell in each year nor each family's worth year by year (L146, "a recorded family run replays with no owners"), which the viewer keeps from the families' yearly reports.

**What could be left out.** Each part in turn was put back at its default in a picture taken at the turn into 1850 and the game played on two years (`--ablate`). 113 parts stop the run at once (a panic, or the money check), 17 make a different economy, 3 leave the economy as it was and change the news, and 78 change nothing in two years: scratch worked out again before it is read (`month_stats`, `bids`, `same_counts`, `pbar`), but also parts read only now and then (`disc_X0`, the discovery thresholds; the tech indices; the founding stream). Two years from one turn cannot tell those apart, and the 78 weigh about 3 MB of 51. So the picture should hold everything.

## 5. Two kinds of save together

The log of orders is the save of record. It is small, a person can read it, it is the only thing that survives a change of the simulation's code, it lets a game be replayed (the timeline's past, an inspector's "what if"), and the check in it says when a game played again is not the game that was saved. A picture of the state is for speed. They live together like this:

- **Where.** Beside `island-7-families-6.json`, a picture `island-7-families-6.state` holding the state at the last year's end, and in it the save's fingerprint of the rules, the crate's version, a fingerprint of the picture's layout (the names and types of its parts), the `check` of the island it holds, and the number of orders in the log it was made from. A picture never holds an order; the log never holds state.
- **When.** At every year's end, after the year's save, on the simulation thread, written as the save is (under another name, then renamed into place). Raw, about 50 MB; with gzip, 9 to 11 MB and about a second (2.4 s here under the load, 0.84 s of it the processor's own time), which at Fastest (two years a second) is too slow to do every year. So: raw and without the books' history at every year's end, or packed every ten years; and the history (the record, the news, the books' past years), which only grows, as an addition. The drive the saves sit on may be a tight one, and three back copies and nine games set aside already sit beside each save; pictures should not be multiplied in the same way. One picture per game, the latest, is enough, since the log can always make another.
- **Loading.** If the picture's version, rules fingerprint and layout match today's, and its order count is no more than the log's, the island is read from it and the orders after it are played as `save::load` plays them today. If its check does not agree with the log played to the same point (which a test does, not every load), something is wrong with the writer, and the log wins.
- **A save made under older rules.** The two kinds then tell different stories, and the player should choose. Played again from the log under today's rules, the island is internally sound but not the island the player left: the towns, the works and the family's fortune drift, and some orders may be refused. Played on from the picture under today's rules, the island is exactly as the player left it and the future follows the new rules, which is how a game expects a patch to behave. The second is only possible when the picture's parts can all be read: a part the picture lacks (a mechanism added since) must be made by that mechanism's own start-up from the state as it stands, and a part whose shape has changed (a seventeenth good) cannot be read at all. The card should say, for example: "This game was saved by an earlier version of the game. **Go on from 1847 as you left it**: the island is as you left it, and from now on the new rules apply. **Play it again from 1750**: your orders are given again under the new rules, and the island will turn out differently (3 of 41 orders cannot be given again)." Where the picture cannot be read, only the second, and the card says why in a sentence. Whichever is chosen, the log keeps every order, and the old save is set aside whole, as it is today.

## 6. A design in stages

| stage | what | cost | what it gives |
|---|---|---|---|
| 0 | This probe and its example. | done | The sizes, what changes when, that a picture is sound, and what is hard. |
| 1 | The streams' state made writable: `rand_pcg`'s `serde` feature, or a count of the words drawn in `Rng`. | an hour | The one thing an outside writer cannot do. |
| 2 | A picture written beside the log, in the core (`snapshot.rs`: the trait and impls of the prototype, the impls for `Family` and `Books` beside them), never read by the game. A test in `tests/save.rs`: at a turn and in the middle of a year, the game gone on with from its picture is the game gone on with from its log, by `check` and byte for byte. | a day | Proof on every build that the picture is whole; a field added to the state without the writer knowing does not compile. |
| 3 | The record keeps who owned each cell and each family's worth by year (`i8` or `i16` per cell, 1.5 MB a century), and the viewer can build its `Board` from a run's record, news and series instead of from years played. | a day or two, mostly the viewer | A game loaded from a picture opens with its past; a recorded family run replays with its owners (L146). |
| 4 | Load from the picture when it matches; the log for everything else. | half a day | Loading a century in tenths of a second instead of 23 s; 2050 no worse. |
| 5 | Trimmed and packed: the works' slots to those used, the record and the series to the years played, the books' history written as it grows, gzip if flate2 is taken on. | a day | 9 to 11 MB a picture instead of 50 to 95; a picture at every year's end without a stall. |
| 6 | Pictures that outlive a change of the rules: parts written by name (as `run.bin`'s arrays are), missing parts made by their mechanism's start-up, a month played on a copy before the game is handed over, and the card of section 5. | two or three days | A game that survives a core landing as the island the player left. |

## 7. What could go wrong

- **A part forgotten.** Guarded at compile time by the struct-literal reader, and at test time by the test of stage 2. A part held in a `static` or a thread-local, outside the state, would escape both; there are none today.
- **A part worked out again on loading that is not a function of what is saved.** The transport is the example, and there may be others that hold a value from before a change of their inputs. Write everything; never rebuild on loading what the run did not rebuild at that moment.
- **Private fields.** Each new private field in `families.rs` or `accounts.rs` must go into its writer; the literal reader catches it.
- **The books.** They are the largest part and nothing in the economy reads them. Without them, a game loaded from a picture would have books only from the year it was loaded, and its accounts panel would start there. Writing their history as it grows avoids writing 44 MB every year.
- **A picture from a build that differs only in the code.** The rules fingerprint sees the numbers and not the code, so the crate's version has to be in the picture and checked; and since helpers rarely bump the version, a fingerprint of the build (a hash of the source at build time, or the git commit) is safer.
- **The disk.** A raw picture is 50 to 95 MB, and the drive the saves sit on may be tight. One per game, packed, or the history written as an addition.
- **The money check.** `money_start` and `issued` must come back to the bit, or the first month after loading fails the check; they do in the prototype.
- **Wall time in the state.** `Books::seconds` is the time the books took; it is harmless, but it means "the same state" has to be defined without it.
- **2050.** The record, the series and the books are all made ready for the whole run at the start and would nearly double; trimming (stage 5) is what keeps a picture of 1800 small in a run that goes to 2050.

## 8. What to build first

Stages 1 and 2 together, as one piece: a module `crates/isostate-core/src/snapshot.rs` with the prototype's writer and reader moved in from `examples/save_state.rs` (the trait, the impls, the list of `State`'s fields), the writers of `Family` and `Member` in the module (their hidden fields are `pub(crate)` already) and of `Books` beside it in `accounts.rs` (or its fields and `Acc` made `pub(crate)`), the streams written by their state through `rand_pcg`'s `serde` feature, and `Sim::picture() -> Vec<u8>` and `Sim::from_picture(&[u8], rules, seed, families)`. One test in `tests/save.rs`: a game of thirty years with orders, pictured at a turn and in June, gone on with from the picture and from the log, the same by `check` and by bytes. Nothing reads a picture in the game yet, so nothing a player sees changes and no fact can move; it can be built at once, but it touches `accounts.rs` (and must list every field of `Family`), where other helpers are at work tonight, and is better done after their branches land. Stage 1 alone (a count of the words drawn kept in `Rng`, or the `serde` feature turned on) changes no number and could land first by itself, but gives nothing until stage 2 uses it. Then stages 3 and 4, which make the load fast, after `night/calendar`'s layout of saves, which has landed.

## How to run it again

```bash
cargo build --release -p isostate-core --example save_state
./target/release/examples/save_state --seeds 7,3                 # sizes, what changes when, round trips
./target/release/examples/save_state --seeds 7 --ablate --no-change   # each part left out in turn
./target/release/examples/save_state --seeds 7 --out DIR         # also writes the raw pictures, to pack from the shell
```

A run of one island with the fingerprints took eight to ten minutes under the load here and should take a few on a quiet machine; the ablation adds some two hundred two-year games.
