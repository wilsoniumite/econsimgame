# dx-record: what the record carries for the viewer, and what it should carry next

Branch `night/o-dx-record`, cut from `main` at 11dbe7d on 12 November 2026. It adds one file, a measuring example, `crates/isostate-core/examples/record_probe.rs`; nothing in the simulation, the record or the viewer changes. Results are under `.claude/night/reports/opus/dx-record/`: the printed tables (`probe.txt`, `probe2.txt`, `probe2050.txt`), a table by year for each island (`probe/seed7/years.csv`, `probe/seed3/years.csv`), every array of each run file with its size (`arrays.csv`), and each thing priced here written out as a whole run of its own, for compressing.

It answers pieces 9, 4 and 17 of [the findings](../backlog/found-along-the-way.md) and Ready next 6 and 14 of [the backlog](../BACKLOG.md): what `run.bin` and the live game hand the viewer, what is lost on the way to the picture, what each missing thing would weigh, and a next layout in stages with the works' rows first and a test that holds the record as the golden test holds the series.

Words. **The record** is what `record::pack` writes as `run.bin` at the end of a run (README, "The export format"). **The hand-over** is what a live game gives the viewer: `live_static` once, `live_year` at every year's end, `live_month` every month (`crates/isostate-core/src/live.rs`); the viewer writes both into the same `Replay` (`crates/isostate-viewer/src/live.rs`, `apply_year`), so that "the map, the panels and the timeline cannot tell a live game from a replay". A **cell-year** is one cell in one year: a layer of one byte a cell is 10,080 bytes a year and 1.52 MB a run of 151 years. A **works-year** is one works in one year.

How it was measured. Islands 7 and 3 under the game's rules with six families, played a month at a time as a live game plays them (`sim::run_month` twelve times, then `sim::end_year`), to 1900, and island 7 again to 2050. After each month and each year the example makes each candidate's bytes from the island as I would propose to encode them, timing the making ("write") and the turning back into what the viewer would hold ("read"), and counts what today's bytes lose or clip. At the end it packs the run, times that, and reads the file back as the viewer does in outline (the header parsed, every array copied out into numbers of its type). With `--hash` it also packs the port's record of seed 7 as the golden test's two runs would make it and hashes every array.

```bash
cargo run -p isostate-core --release --example record_probe -- --out DIR --hash 7 3
cargo run -p isostate-core --release --example record_probe -- --end-year 2050 7
```

The machine was shared with some thirty other workers building and running all the while. Sizes and counts are exact, and the two runs to 1900 agree on every one of them to the byte; times moved by up to a half between the runs (packing island 7 took 235 ms in the first and 160 in the second), so read them as orders of size. Compressed sizes are gzip's (PowerShell's `GZipStream` at its best setting, and Python's `gzip` at 6 for the layers cut out of a run file), since the web viewer's runs travel gzipped.

## In short

1. **A game's record is 47 MB on island 7** (17 MB gzipped), and half of it is the books of account, which the viewer does not read. The nine map layers are 13.7 MB, the three town layers 4.6 MB, the works' capacities 0.3 MB. It is packed in 0.1 to 0.25 s and read back in 0.04 to 0.09 s. The port's record of the same island is 20 to 22 MB. A live year hands over about 245 kB in 1 to 2 ms, and a month 4 kB.
2. **The 3D viewer reads 34 of the 63 arrays.** It reads none of the books (14 arrays), the harvest, the towns' knowledge and income ratio, the navigable rivers or the seven resource fields; of the header it never reads the methods' recipes, the canals, the railways building, the techs or the series' labels. It never checks an array's declared type (K5), and its own test writes types ("u1", "f4") that the core never writes ("uint8", "float32").
3. **The land is two classes where it is three or four.** On average 20 to 23 per cent of a land cell's ground lies outside the two classes recorded, in every year. In two land cells of three the third class holds a fifth of the cell or more. Three classes keep all but 0.5 to 1.5 per cent; four keep all but a tenth of one.
4. **Of the bytes said to clip, only rent does before 1900.** Rent passes the byte's top of £1,000 a month in 221 cells of island 7 and 229 of island 3 in 1900, some 5,200 cell-years a run. The way to market never comes near its top: the farthest cell's path cost is 13 in 1900 against the byte's 63.75, and no land cell is without a way. Residents reach 1,417 a cell against the byte's 2,040. Run on to 2050, rent is at the byte's top in 4,743 cells, 316 cells hold more residents than the byte can say, and 542 cells that joined their town after 1900 all read as joining in 1900.
5. **The works' rows.** Today a recording gives the viewer a works' capacity year by year and nothing else, and a live game the same. The books in the same file keep 21 numbers a works-year (6.1 MB), and nothing reads them. What the site planner waits for comes to five bytes a works-year beside today's capacity: about 0.34 MB a run (0.20 MB gzipped), 0.7 per cent of the file, made in 21 microseconds a year. The stock in a works' yards and the fuel it burns, which the planner also asks for, the simulation does not keep for a works at all (stage 1 says what can stand in). Live, about 6 kB more a year, and 5.8 kB a month for the month's hands and work.
6. **"Going up" at a year's end is what the turn itself ordered.** In every quarter-century year from 1750 to 1900, 0 to 13 works are still building as December ends; right after the turn 55 to 214 are. A works orders its growth at the turn and has it delivered over the next year, so the share on order recorded at the turn is the next year's scaffolding, not the last year's.
7. **A closed works needs no rows.** Its year of closing is in the works' table and its last capacity in the series. Six years of rows after closing would add 13 per cent more rows and nothing the simulation knows.
8. **Owners and traffic are cheap to record.** The squares families own, as rows a year, are 5 kB a run gzipped; a live game today sends all 10,080 squares' owners every year (20 kB) for the 2 to 6 that change hands. The weight carted past each cell, a byte a year on a log scale, is 1.5 MB a run and 0.64 MB gzipped; by boat 0.05 MB. Month by month the carting would cost the simulation's thread 2.6 ms a month and 36 MB a run: not for the record.
9. **A test that holds the record** costs half a second to two seconds on top of the golden test, if it hashes every array of the port's record in the golden test's own two runs. The game's record moves with every rule, so it is held by a contract instead (each array's name, type, shape and whether a byte saturates) and by a test in the viewer that a live game and its recording are read alike.
10. **Five stages**, each half a day to two days: the test and the works' rows; who owns what; truer bytes for the land; traffic; what the picture cannot show yet. Build the first first: the site planner is waiting on it, and the test protects every stage after.

## 1. What the record carries, and who reads it

### The run file

Island 7, the game's rules, six families, to 1900: 63 arrays (`arrays.csv`). "Reads it" names the viewer's files that use the field, found by searching `crates/isostate-viewer/src` for each (`replay.rs` and the tests left out). The web viewer is `viewer/index.html`, which the Python builds; the Python's `record.py` writes the port's arrays by the same names and encodings.

Three scripts read the file too, and a new layout has to keep them working: `crates/isostate-core/parity/check_runbin.py` holds a Rust `run.bin` to the layout of a Python one (every array's name, type and shape, the header's keys, the alignment), so anything the port's run writes that the Python does not is a failure there; `parity/check_facts.py` works the sweep's facts out again from the file, among them the works' sizes from `firm_first`, `firm_len`, `firm_off` and `firm_Kvals`; and `notebooks/explore.py` reads any array by its type through numpy. In the core, `tests/run.rs` holds that two identical runs pack to the same bytes, that five arrays sit on eight bytes, that `use` has its shape and the recorded bytes, and that the mix's classes and shares are in range. None of these holds what an array says, which is R2.

| array | type, per | bytes a run | the 3D viewer reads it in | web viewer | what is lost |
|---|---|---|---|---|---|
| `elev`, `terrain`, `river`, `down` | uint8 / int32, a cell | 70 kB | `world.rs`, `rivers.rs`, a dozen others for `terrain` | yes | |
| `nav` | uint8, a cell | 10 kB | nowhere (rivers are drawn from `river`) | no | |
| seven resource fields | uint8, a cell | 71 kB | nowhere | no | |
| `use` | uint8, cell-year | 1.52 MB | `terrain.rs`, `scatter/towns.rs` | yes | |
| `inten` | uint8, cell-year | 1.52 MB | `terrain.rs`, `scatter/towns.rs` | yes | on farmed land, clipped at 45 hands a cell (not counted) |
| `rent` | uint8, cell-year, log 0.001 to 1,000 a month | 1.52 MB | `overlay.rs` | yes | stops at £1,000 a month (L131) |
| `trans` | uint8, cell-year: road 0 to 3, +4 rail, +8 rail building, +16 canal, +32 canal digging | 1.52 MB | `rails.rs`, `network.rs`, `course.rs`, `world.rs`, `scatter.rs`, `scatter/towns.rs` | yes | the canal bits are read nowhere |
| `catch` | uint8, cell-year: town + 1 | 1.52 MB | `overlay.rs`, `family.rs`, `main.rs`, `scatter/towns.rs` | yes | |
| `dist` | uint8, cell-year: quarters of path cost to 63.75, 255 for none | 1.52 MB | `overlay.rs` | yes | 63.75 and "no way" share a value (L132): never met to 1900 |
| `mix_a`, `mix_b`, `mix_s` | uint8, cell-year: two largest of 12 classes and the first's share | 4.57 MB | `world.rs`, `main.rs`, `scatter/towns.rs` | yes | a fifth of the land's ground (R157, L16) |
| `built`, `res`, `works` | uint8, cell-year (kept town ground) | 4.57 MB | `world.rs`, `scatter/towns.rs`, `overlay.rs` | no | the year joined stops at 1900, residents at 2,040 (R104) |
| `town_active`, `town_pop`, `town_urban` | uint8 / float32, town-year | 65 kB | `main.rs`, `boats.rs`, `network.rs`, `shots.rs`, `world.rs`, `scatter/towns.rs` | yes | |
| `town_wage`, `town_price` | float32, town-year (and good) | 493 kB | `overlay.rs` | yes | |
| `town_y` | float32, town-year: the year's mean income ratio, which the people's growth answers to | 29 kB | nowhere | yes | not in the live year (L196) |
| `town_know` | uint8, town-year-tech | 152 kB | nowhere | yes | |
| `firm_first`, `firm_len`, `firm_off`, `firm_Kvals` | int32 a works, float32 a works-year from its first year with plant to its last | 293 kB | `scatter.rs`, `network.rs`, `family.rs`, `game.rs`, `main.rs`, `shots.rs` (by `replay::firm_k`) | yes | everything but capacity |
| `series` | float32, series-month | 790 kB | `chart.rs`, `ui.rs`, `clock.rs`, `overlay.rs` (by `yearly`, `monthly`, `annual`) | yes | |
| `town_harvest`, `harvest` | float32, town-year and year | 30 kB | nowhere | no | C65 |
| `mine_*` (11 arrays) | a row for every working there is or has been, every year | 2.57 MB | `scatter.rs` reads seven columns | no | `left`, `cells`, `royalty` unread |
| `acct_*` (14 arrays) | the books: towns, goods, land, methods, needs, flows between towns, uses, and 21 float32 columns a works-year (among them its owning family, hands and how hard it ran) | 22.37 MB | nowhere | no | |

The array types written today are `uint8`, `int32` and `float32`. The web viewer reads every array by its declared type through a table of six (`uint8`, `int32`, `float32`, `uint16`, `int16`, `float64`; `CTORS` in `viewer/index.html`), and an array of any other type throws there and the run does not open. So a new array must be one of those six: never `int8`.

The header is JSON, 1.57 MB on island 7: the island's size and dates, the goods, the methods with their inputs, heat, labour and usual size, the techs with when and where each was found, the sectors, the land uses and display classes, the towns, every works with the sums that put it there, every news item with its town and cell, the railways opened and building, the series' names, labels and groups, the canals' links, and the books' columns and the island's account by year. The 3D viewer reads the size and dates, the goods' and methods' names and labels and each method's good and usual capacity, the towns, the works and the news (`replay.rs`, `Header`). It reads no method's recipe, though the header has it (the site planner's hand-over asks for exactly that), and no canal, railway building, tech, or series label.

### The live hand-over

`LiveStatic` (334 kB as JSON, once) carries the island, the goods, the methods (name, label, good, usual capacity, group: no recipe), the techs, the sectors, the names of the layers and series, and the new land. `LiveYear` carries the same layers as the record, the towns' active, people, townspeople, wage, prices and knowledge (no income ratio), each standing works' capacity, the works and towns new since the last year and the works closed, the railways building, the news and changed marks, the year's series, every family's full report, the owner of every cell as `i16`, the harvest by town and island, the workings, the canals' links, and the island's account. `LiveMonth` carries the month's series, wages, prices, families' cash and news.

The viewer's `apply_year` keeps the layers, the town tables it reads, the works, the towns, the news, the books' year series, the workings and the owners; the families go to their own view. It drops the knowledge, the harvest, the canals, the railways building and the island's account.

### What is lost between the simulation and the picture

- **The land's mix** (R157, L16, L35). Two classes and a share a cell, where the picture blends four cells and the ground is often three classes; the inspector names a cell's two.
- **Rent above £1,000 a month** (L131): every busy town centre reads alike.
- **The way to market** past 63.75 (L132): one value for far and for no way. Not met on these islands.
- **Residents above 2,040 a cell and the year a cell joined its town after 1900** (R104): met only in a run past 1900.
- **Traffic** (L192, L166): the weight carted past each cell (`State::traffic`, worked out once a year in `sim::update_roads`) and boated (`State::cn.traffic`) is in no layer, so nothing that moves can follow it and no map view can show it.
- **Who owns what in a recording** (L146): `run.bin` has no cell owners, and its header's works have none, so a recorded family game replays with no pennants or fences. (Under the game's rules each works' owning family is in the books, `acct_works` column `owner`, year by year; nothing reads it.)
- **The works**: capacity only, where the site planner wants how busy, how many hands, what is on order and its state, and draws stand-ins (R140, L7, L67; the works-sites hand-over, item 3). The books have hands and work and the viewer does not read them.
- **The machines** (C24): `State::machine_stock` (each town's share of each sector worked by machines) and their quality are recorded nowhere.
- **The harvest** (C65): recorded by town and island, read nowhere; the yield of each cell (`State::harvest_mult`) is recorded nowhere.
- **Where a news item happened** (K4): carried in the record and the hand-over, used nowhere. No bytes are wanted.
- **The towns' income ratio** in a live game (L196), and the methods' recipes in a live game (works-sites hand-over, point 2; K34).
- **An enclosure year** (L175): the core keeps none, so this is a rule to make, not a byte to record.
- **Whether a byte means what the reader thinks** (K5): no type is checked, and no test notices a change to any array (R2).

## 2. What each missing thing would weigh

### Today's record and hand-over

| | island 7 | island 3 |
|---|---|---|
| `run.bin`, raw and gzipped | 46.78 MB, 16.71 MB | 43.69 MB, 15.05 MB |
| of it: the books | 22.37 MB (48%) | 19.29 MB (44%) |
| the nine map layers / the three town layers | 13.70 / 4.57 MB | 13.70 / 4.57 MB |
| the workings | 2.57 MB | 2.82 MB |
| the header | 1.57 MB | 1.35 MB |
| packed in, read back in (two runs) | 235 and 160 ms; 74 and 50 ms | 203 and 111 ms; 93 and 42 ms |
| `live_year`: bytes, a year's mean (1750, 1900) | 244 kB (275, 289) | 242 kB (256, 276) |
| `live_year`: time to make, median and longest | 1.4 ms, 27 ms | 1.4 ms, 36 ms |
| `live_month` | 4.2 kB, under 0.01 ms | the same |
| the map's bytes (`record::layers` and `town_layers`) | 0.9 to 1.7 ms a year | 0.9 to 1.5 ms |

Of today's layers gzipped (island 7): `use` 48 kB, `trans` 47 kB, `catch` 35 kB, `dist` 85 kB, `mix_a` 74 kB, `mix_b` 96 kB, `mix_s` 489 kB, `rent` 592 kB, `inten` 707 kB; `built` 11 kB, `res` 49 kB, `works` 59 kB. The capacities `firm_Kvals` hardly compress (265 kB to 238 kB), nor the books' works table (5.86 MB to 4.09 MB).

The port's record of seed 7 to 1900, as the golden test's runs make it: 20.30 MB with no families and 22.28 MB with six, 33 arrays. Packed in 309 and 1,175 ms; every array and the header (without its date) hashed in 146 and 514 ms. The header with six families is 4.9 MB, nearly all of it the works' reasons.

### What today's bytes lose or clip

Islands 7 and 3 (5,846 and 5,842 land cells), from `years.csv`:

| year | ground outside two classes | outside three | land cells with a third class of a fifth or more | cells at rent's top (7 / 3) | the dearest cell, a month | farthest path cost (byte's top 63.75) | fullest cell (byte's top 2,040) |
|---|---|---|---|---|---|---|---|
| 1750 | 0.206 / 0.203 | 0.005 / 0.005 | 3,728 / 3,740 | 0 / 1 | 470 / 1,392 | 50.8 / 45.5 | 518 / 519 |
| 1800 | 0.223 / 0.226 | 0.005 / 0.006 | 4,005 / 4,056 | 2 / 1 | 1,033 / 1,038 | 35.8 / 34.6 | 792 / 761 |
| 1850 | 0.228 / 0.229 | 0.005 / 0.015 | 4,005 / 3,902 | 13 / 13 | 1,462 / 1,311 | 22.9 / 17.2 | 1,119 / 1,117 |
| 1875 | 0.213 / 0.224 | | | 86 / 74 | 2,068 / 1,638 | 20.0 / 13.4 | 1,252 / 1,269 |
| 1900 | 0.205 / 0.205 | 0.005 / 0.009 | 3,789 / 3,679 | 221 / 229 | 2,436 / 2,675 | 12.8 / 13.2 | 1,417 / 1,409 |

Outside one class, the share is 0.52 to 0.58; outside four, 0.000 to 0.001. Rent is at the byte's top in 5,216 cell-years of island 7 and 5,174 of island 3, from 1795 and 1750. No land cell is without a way to a market in any year. (The shares outside one, three and four classes and the cells with a third class of a fifth were read from `mix12.bin` with a few lines of Python; the rest is printed by the example.)

### What each would weigh

Bytes in 1900, the whole run raw and gzipped, and the mean time a year to make the bytes and to read them back, island 7 (island 3 within a few per cent unless shown):

| candidate | 1900 | run, raw | run, gzipped | write, µs a year | read, µs a year |
|---|---|---|---|---|---|
| `mix4`: four classes and three shares, 7 bytes a cell | 70,560 | 10.65 MB | 2.61 MB | 506 | 96 |
| `mix3`: three classes and two shares, the two bytes it adds | 20,160 | 3.04 MB | 0.57 MB | (as `mix4`) | |
| `mix12`: all twelve shares | 120,960 | 18.26 MB | 2.42 MB | 435 | 82 |
| `rent16`: two bytes, log 0.001 to 10 million | 20,160 | 3.04 MB | 1.65 MB | 94 | 103 |
| rent's cells past the byte, as rows of cell and rent | 1,768 | 42 kB | | | |
| `dist16`: hundredths, 65535 for none | 20,160 | 3.04 MB | 0.16 MB | 114 (23) | 10 |
| `res16`: people | 20,160 | 3.04 MB | 0.07 MB | 37 | 6 |
| `built16`: the year | 20,160 | 3.04 MB | 0.02 MB | 9 | 2 |
| `traffic16`: carted past each cell, two bytes on a log scale | 20,160 | 3.04 MB | 1.74 MB | 91 | 193 |
| the same in one byte | 10,080 | 1.52 MB | 0.64 MB | | |
| `boats16` (one byte: 1.51 MB, 0.05 MB gzipped) | 20,160 | 3.02 MB | 0.11 MB | 31 | 12 |
| `owner8`: a byte a cell | 10,080 | 1.52 MB | 0.014 MB | 6 | 4 |
| `owner_rows`: cell and family for each square owned | 2,856 (3,615) | 0.32 MB (0.42) | 0.005 MB | 25 | 5 |
| `owner_moves`: the squares that changed hands | 6 (9) | 1.6 kB | 1.3 kB | 26 | 4 |
| `works12`: twelve bytes a works-year, from its founding, kept six years after closing | 10,824 (10,284) | 0.95 MB (0.79) | 0.57 MB (0.48) | 21 | 1 |
| `works_month`: id, hands, work and state, eight bytes a works-month | 69,528 a year (5.8 kB a month) | 6.51 MB | 3.29 MB | 195 | 7 |
| `machines`: towns by sectors by two uses, and the quality | 676 | 0.10 MB | 0.016 MB | 8 | 0 |
| `harvest_cells`: farms' and pastures' yield, 128ths of an average year | 20,160 | 3.04 MB | 0.28 MB | 66 | 13 |
| `town_y` in the live year | 192 | 29 kB | | 1 | 0 |
| `traffic_month`: the month's carting laid on the paths, two bytes | 20,160 a month | 36.5 MB | 20.9 MB | 31,000 (2.6 ms a month) | 1,500 |

On the works (islands 7 and 3): 78,772 and 65,563 rows a run with the closed kept six years, of them 68,263 and 57,938 for works standing. In 1900 island 7 has 749 works standing; as the turn leaves them, 152 are going up (more than 5 per cent of their plant on order, `works::building`), 480 work above half their capacity, 97 between a half and 0.15, 20 below, 15 closed that year and 138 closed in the six years before. As December ends, before the turn, 0 to 13 works are still going up in each of the quarter-century years shown: the 55 to 214 after it are the turn's own orders.

### Run to 2050

Island 7 again, to 2050 (`probe2050.txt`; 400 s of simulation played month by month). The record is 112 MB, of which the books are 61 MB; it is packed in 0.57 s and read back in 1.06 s. What clips:

| year | cells at rent's top | the dearest, a month | cells past the residents' byte (over 2,044) | the fullest cell | cells joined after 1900 (all read as 1900) | farthest path cost |
|---|---|---|---|---|---|---|
| 1925 | 359 | 3,483 | 0 | 1,561 | 43 | 9.0 |
| 1950 | 538 | 4,354 | 0 | 1,702 | 112 | 8.7 |
| 2000 | 1,569 | 4,958 | 0 | 2,026 | 272 | 8.5 |
| 2025 | 3,028 | 11,059 | 42 | 2,149 | 373 | 8.5 |
| 2050 | 4,743 | 21,205 | 316 | 2,330 | 542 | 8.5 |

So in a run to 2050 rent's byte is at its top on four land cells in five by the end, residents clip from some year between 2000 and 2025, and the year joined is wrong for every cell added after 1900 (R104, and the 2050 design's need of "a residents byte that does not clip"). The way to market never comes near its top. Works standing grow to 2,210 in 2050 (9 kB of capacities a year), and the works' rows of twelve bytes would be 3.6 MB a run. The live year's mean grows from 1.4 to 8.9 ms, nearly all of it a few long years; its bytes from 245 to 280 kB.

## 3. The next layout

### What every stage keeps to

- **The port's arrays stay exactly as they are**, by name, type and encoding, under both sets of rules. The Python writes them, the web viewer reads them, `parity/check_runbin.py` holds the port's layout to the Python's, and the port's record is to be held byte for byte (stage 1). What is new is added beside them, never in their place, and only in a run whose rules have it, which is the convention every rule follows: a switch, off under `default_params` and on under `game_params`. One switch for the record will do, `record.next`, since the record only watches; each stage adds to what it writes.
- **A name is never given a new meaning.** A widened layer gets a name of its own; the byte layer goes on being written beside it while the web viewer reads it.
- **Every new array is one of the six types the web viewer knows**, and is aligned as today's are.
- **The reader finds by name and does without.** The 3D viewer already reads `rent`, `dist`, the town layers and the mix only if a run has them, and falls back or says so. Every new array is read the same way, so a run recorded today goes on opening, and what it lacks is drawn as today (stand-ins for the works, no pennants in a replay). The header's `meta` gains `"layout": 2` for a run with the next layout (absent means today's), so the viewer can say "this run was recorded before the works' rows" rather than guess.
- **The reader checks each array's type** (K5) against what it expects, and says which array and what it found. The viewer's own test fixture writes "u1", "i4" and "f4"; it is put right to the core's "uint8", "int32" and "float32".
- **One function makes each part for both.** Today `record::layers` makes the map's bytes for the record and for `live_year` alike, and that is why a live game and a recording agree. Each new part is made the same way, by one function that `record::snapshot` and `live_year` both call.
- **Saved games are not touched.** A save is the seed, the rules and the player's orders, played again from 1750 when it is loaded (`save.rs`); it holds no record, and the record is made again as the years are played. So no layout of the record can break a save, and none of this needs a step in the save's version. One thing does show: a save names its rules by a fingerprint of every parameter (`save::rules_id`), so the switch `record.next`, once it is in `Params`, moves the game's fingerprint, and when a game saved before stage 1 is loaded the viewer prints "it was saved under other rules ..., so the years may play out differently", though not one year plays differently. Every new rule does the same today; if that is unwanted for a switch that only watches, it can be left out of the fingerprint.
- **The month by month hand-over** carries only what changes in a month and what the viewer would show by the month. A recording keeps years. What a live game is told by the month the viewer keeps beside the run, as `live::Months` keeps the months' wages and prices today, and a replay shows the year.

### Stage 1: the record's test, and the works' rows

*What it gives.* The site planner on `night/works-sites` draws a works' busyness and stock from stand-ins (the hand-over's item 3: busy 0.9 in a year its capacity rose, 0.45 when it fell, else 0.75). With these rows it draws its busyness from what the works did, in a replay and in a live game alike, and month by month in a live game; its stock it can draw only from a stand-in that is nearer the truth (see "What the rows cannot give" below). Scaffolding follows the share on order, which is next year's growth. The test protects this stage and every one after it. Rows answered: R2, K5, R140; L7 and L67 in part (the site planner's half is its own); piece 4's half for the record.

*The test.* Three parts.

1. **The port's record, held byte for byte, in the golden test's own runs.** `golden()` in `tests/golden.rs` already returns the run. Pack it, hash every array and the header (with `meta.generated` taken out) with the same FNV-1a, and hold each to a constant, as the series and the news are held, with the same `ISOSTATE_GOLDEN_PRINT` and `ISOSTATE_GOLDEN_DUMP`. A hash for each array, so a failure says which array moved: "`mix_s` differs" is a different story from "the header differs". It costs 0.45 s on the run without families and 1.7 s on the run with six (packing and hashing, measured above), against the 31 and 34 s each run already takes. The constants the example printed today are in `probe.txt`.
2. **The game's record, held by a contract.** The game's record moves whenever a rule moves, so a hash would be retaken at every landing and hold nothing. A test in the core instead runs seed 7 under the game's rules to 1760, packs it, and checks every array against one table of the layout kept in `record.rs` (name, type, shape, what it is): every array written is in the table and every one in the table that the run's rules call for is written, its type is one of the six, its bytes are its shape's, it starts on eight bytes. And no byte layer or row column saturates: the count of values at a byte's top is printed and held to none for the layers that have a wider twin. A run to 1760 with six families is two or three seconds.
3. **A live game and its recording, read alike, in the viewer.** `live_tests.rs` already holds that a game played and the same game loaded from its save are the same in the viewer (`same`). The new test plays a game to 1760 straight through `sim::run_year` and `live_year` into a `Replay` as the thread does, packs the same run, reads it with `replay::parse`, and holds the two `Replay`s equal, array by array, through `same`, which is widened to every field both have (rent, way to market, wages, prices, the town layers, the workings, the works' rows). Where one has what the other has not (owners until stage 2), the test says so in a list it checks, so the gap is named and closes when it is filled. Two things in `same` have to be compared another way than today, where both sides are live games: a live game gives every works a whole row of capacities from 1750 (`firm_first` 0, `firm_len` the run's years) and a recording only the years it stood, so capacities are compared through `replay::firm_k` works by works and year by year, not as arrays; and the works' owners, which `same` compares, are in a live game's works and not in a recording's header (stage 2 puts them there).

*The works' rows, in the run file.* Under `record.next` the works' series `firm_first`, `firm_len`, `firm_off`, `firm_Kvals` run from the year a works is founded (with no plant yet, so capacity 0) to its last year standing. Today's readers take capacity 0 as "not there yet", as they do now. (This bends the rule above that a name keeps its meaning: `firm_first` becomes the year founded where it was the first year with plant. It is safe, since every reader of the arrays, the 3D viewer's `firm_k`, the web viewer's `firmK` and `check_facts.py`, treats a capacity of 0 as no works, and a live game already lays its works out from 1750 with zeros before each is founded. One thing to mend with it: `check_facts.py` takes the last capacity of each works standing at the end as its size, and a works founded in the last year with no plant yet would enter its median at 0; it should leave out capacities of 0.) Beside `firm_Kvals`, row for row, four columns:

| array | type | what |
|---|---|---|
| `firm_order` | uint8 | the share of capacity built and on order that is still on order at the year's turn, in 255ths: the coming year's scaffolding |
| `firm_hands` | uint16 | hands, the year's mean, in tenths (to 6,553) |
| `firm_util` | uint8 | how hard it ran: output over capacity, the year's mean, in 255ths |
| `firm_state` | uint8 | bit 0: going up by the works' own rule (`works::building`) at the turn; bits 1 and 2: years in a row at a loss (`f_loss_years`, 0 to 2; at three it closes) |

Five bytes a works-year on top of today's four: about 0.34 MB a run on island 7 (68,263 works-years from founding to the last year standing, 2,062 of them founding years with no plant, which today's capacities leave out and which add 8 kB of capacities), 0.7 per cent of the file. Gzipped, the four columns come to 0.20 MB on island 7 and 0.17 MB on island 3 (measured in the check of this probe, from the twelve-byte rows cut to the works standing and laid out works by works, column by column; its state byte was the example's, not the bits above). The twelve-byte row measured above, with room for yards and kept for the closed, is 0.95 MB and 0.57 MB gzipped; this is lighter on both counts. Made in about 21 µs a year. Hands reach 1,639 and 1,722 at the most on islands 7 and 3 to 1900, well inside the column's 6,553.

*What the rows cannot give.* The site planner's hand-over also asks for the stock of each input and of the output in a works' yards at the year's end, and the fuel it bought. The simulation keeps neither for a works: the stocks are the town's (`State::inv`, by town and good, the merchants'), and a works' mix of fuels is worked out every month from the fuels' prices at its gate (`econ.rs`, `hshare`) and kept nowhere. The nearest the core comes is `f_fill`, the share of its inputs and fuel a works could get, a running mean; one more byte a works-year (68 kB raw) would carry it, and it would stand in for "how full the yards are" more truly than a rise or fall of capacity does. The fuel's kind can be read from the town's recorded prices only by working the mix out again in the viewer, which would be a second copy of a rule; better a byte for the coal share, if the picture wants it.

Hands and work are the year's means, which the books keep (`acct_works`, columns `hands` and `util`), and the books close before the year's snapshot is taken (`sim::end_year`). Under the game's rules the books are kept; where a run keeps none, the rows take December's hands and the works' own running measure of its work (`f_u`, a mean with a memory of about ten months), and the header says which.

This changes two things in what `dx-combines.md` designed. *State* says only what the simulation alone knows, its own rule for going up and how near the works is to closing; whether a works counts as on short time or standing is a threshold on `firm_util` that belongs to the picture. And *no rows after a works closes*: the year it closed is `died` in the works' table, its size is its last capacity, and the simulation knows nothing more of it (it gives up its share of the cell at once, and marks the cell derelict for six years only if the cell is otherwise empty: `firms.rs`, `derelict_timer`). How long a ruin stands is the viewer's rule, as it is on the works-sites branch today (cold two years, derelict to six, then cleared). The yards and what is passed across a yard come with the mechanisms that make them true (Ready next 11), each as a column of its own.

*The hand-over.* `LiveYear::firm_K` (an id and a capacity, 16 bytes a works) becomes `works`, a row for each works standing (id, capacity and the four above), about 24 bytes a works in memory: 18 kB a year in 1900 where it is 12 kB. `LiveMonth` gains `works`: for each works standing, its hands that month, its work (`f_u`), its share on order and its state, eight bytes: 5.8 kB a month in 1900, 0.016 ms. The viewer keeps the newest month's rows beside the run, as it keeps the months' prices, and the site planner reads them in a live game and the year's rows in a replay.

*The method's recipe.* `LiveStatic`'s methods gain their inputs, heat and labour, as the run file's header already has them (the plant a method needs, which the site planner's hand-over also names, is in neither and would be added to both), and the viewer's `Method` reads them in both. Then the site planner can plan a works from what the running simulation says it uses rather than from the shipped content (the works-sites hand-over, point 2), which matters the day `night/tree`'s thirty new trades land.

*Effort.* Half a day in the core (the switch, the row's function, the run file, the two hand-overs, the layout table, the contract test, the golden hashes), half a day in the viewer (reading with types checked, `Replay`'s new fields, `apply_year` and `apply_month`, the test), and the site planner's reading of it on its own branch.

### Stage 2: who owns what

*In the run file:* `owner_first` (int32, a year), then for each year the squares a family owns, `owner_cell` (uint16: the island's 10,080 cells fit) and `owner_family` (uint8): 0.32 MB raw on island 7, **5 kB gzipped**, since the rows hardly change from year to year. And each works' owner in the header's works table, a number or null, as the viewer's `Firm` already expects (`owner`). *In the hand-over:* `LiveYear::cell_owner` (all 10,080 squares as `i16`, 20 kB a year) becomes the squares that changed hands since the last call, three bytes each: 2 to 6 a year at the turn, and those bought by an order in the middle of a year with the month that bought them (`LiveMonth`). It gives pennants and fences in a replay of a family game (L146), a tint or map view of who owns the land (L6), and cuts part of what the live year resends (L208). Half a day.

### Stage 3: truer bytes for the land

*The mix:* two more layers, `mix_c` (the third class) and `mix_t` (the second class's share), so a cell is three classes and two shares, and the third's share is what is left. 3.04 MB raw, **0.57 MB gzipped**, 3.4 per cent of the gzipped file, and it keeps all but 0.5 to 1.5 per cent of the land's ground where two keep 77 to 79 per cent. Four classes would keep the last per cent for 7.6 MB more raw; not worth it. The ground's shader and the parcels that read the mix (`world.rs`, `terrain.wesl`) learn a third class, and the inspector names three (L35). *Rent:* the byte stays; the cells at its top get a row each year of cell and rent (`rent_over_first`, `rent_over_cell`, `rent_over`, float32), some 5,200 cell-years and 42 kB a run, and the map view reads the true rent there. A second rent layer of two bytes would be 1.65 MB gzipped for the same 5,200 values. In a run to 2050 the rows grow to some 230,000 cell-years, about 1.8 MB (reckoned from the counts at each quarter century, not measured), still under a third of a second layer's 6.1 MB. *The way to market:* nothing; it never clips on these islands. If the new land makes it clip, the same rows. *Residents and the year joined:* two-byte layers, `res_w` (people) and `built_w` (the year), but only in a run that goes past 1900, where the bytes stop; to 1900 they never clip. They compress to almost nothing (0.07 and 0.02 MB a run). It gives R157, L16, L35, L131 and R104. A day in the core and the viewer, most of it the ground's shader.

### Stage 4: traffic

*In the run file:* `traffic` and `boats`, a byte a cell-year each on the rent byte's kind of log scale (ten decades, a step of 9.4 per cent): 1.52 MB raw and 0.64 MB gzipped for the carts, 0.05 MB gzipped for the boats. Two bytes would be 1.74 MB gzipped for a precision no cart needs. *In the hand-over:* the same in `LiveYear`, 20 kB a year. Not month by month: the simulation lays its carting on the paths once a year (`sim::update_roads`), and doing it every month costs 2.6 ms of the simulation's thread a month and 20 kB a month, for a carting that has no seasons yet (the island has no harvest month, F20). If a harvest month comes, the month's traffic can go to a live game then, and a recording keeps the year's. It gives carts, trains and boats that follow the traffic (L166), a map view of it (L192), and the carting the inspector shows on its flows (Ready next 7). Half a day in the core; the drawing is its own piece.

### Stage 5: what the picture cannot show yet

Each small, each closing a row of piece 17:

- **Machines** (C24): `machines`, each town's share of each sector worked by machines (uint8, towns by sectors by two uses) and their quality (float32) a year: 0.10 MB raw, 16 kB gzipped. A cell's machinery is its town's share for the cell's sector.
- **Harvest** (C65): first the town's and the island's, which are recorded already and read by nobody; the viewer can tint a district's fields by them at no cost in bytes. A cell's own yield (`harvest_mult`, farms and pastures) is 3.04 MB raw, 0.28 MB gzipped, if the district's is not enough.
- **Where a news item happened** (K4): no bytes. The cell is in every item; the news panel can take the camera there.
- **The towns' income ratio in a live game** (L196): 192 bytes a year in `LiveTowns`. The knowledge the live year already carries and `apply_year` drops can be kept the same way, for the web viewer's view of it.
- **An enclosure year** (L175): a rule for the core, not a byte; once there is one, a byte a cell, written once a year.
- **Walls of what is quarried, burnt or felled nearby** (L224): no bytes. The workings by kind and cell, the works' methods and cells, and the mix's woods are all in the record already; the houses' generators would read them.

The last row of piece 17, stopping to change a thing and compare, branches of a run and an agent's timeline (R164), is the inspector's (Ready next 3) and is left to it here.

Together, about 0.3 MB gzipped (with the cell's harvest) and a day.

### What each stage costs and gives

Island 7 to 1900, added to today's 46.78 MB (16.71 MB gzipped):

| stage | raw a run | gzipped | live, a year / a month | write, a year | gives | days |
|---|---|---|---|---|---|---|
| 1. the test and the works' rows | 0.34 MB | 0.20 MB (twelve bytes: 0.57) | +6 kB / 5.8 kB | 21 µs; the test 0.5 to 1.7 s | the site planner's real figures; R2, K5, R140 | 1 |
| 2. who owns what | 0.32 MB | 0.005 MB | -20 kB / a few bytes | 25 µs | pennants in replays, an owners' view; L146, L6, L208 in part | 0.5 |
| 3. truer bytes for the land | 3.08 MB | 0.6 MB | +20 kB / none | about 500 µs | three land classes, true rent; R157, L16, L35, L131, R104 | 1 to 2 |
| 4. traffic | 3.04 MB | 0.69 MB | +20 kB / none | about 120 µs | carts and boats on the traffic, a traffic view; L166, L192 | 0.5 |
| 5. what cannot be shown | 3.1 MB | 0.3 MB | +1 kB / none | about 75 µs | machines, harvest, news place, income ratio; C24, C65, K4, L196 | 1 |
| all five | 9.9 MB (+21%) | 1.8 MB (+11%) | | under a millisecond | | 4 to 5 |

The record's half of the books (22 MB raw, 6 MB of it the works' table) is larger than all five together. If the file's size ever matters, that is where to look, and not here.

## 4. What to build first

Stage 1, in one branch: the record's test first (the golden hashes of the port's record, the layout table and its contract test, the viewer's test that a live game and its recording are read alike, the type check), then the works' rows and the methods' recipes in `LiveStatic` on top of it. It is about a day, it is the one the site planner on `night/works-sites` is waiting for, and the test it brings is what lets stages 2 to 5, and every later branch that adds to the record, land without anyone checking the record by hand. Stage 2 is small and next. Stages 3 to 5 can go in any order behind it; stage 3 wants the ground's shader, so it is best taken with the drawing of the new land (Ready next 2), which works in the same files.

## 5. What this probe did not do

- It changed nothing but the example. No layout table, switch, test or row was written; the numbers above are of encodings made inside the example, not of a record that has them.
- Two islands. The sizes scale with cells and years and hardly with the island; the counts (rent's clipped cells, the works) will differ by island.
- The viewer's own reading was not timed: the example reads the run back as the viewer does in outline, without its fallbacks and checks. Neither was the drawing of anything proposed here.
- The intensity byte (`inten`, on farmed land clipped at 45 hands a cell) was not counted, nor whether `catch` can pass 254 towns (`T_max` is 48 today).
- The workings' table repeats every working ever opened in every year after (69,413 rows on island 7, 2.57 MB, 37 bytes each); a table of workings with their years and a row a year only while at work would be smaller. Noticed, not measured further.
- Under the new land (`land.enabled`) none of this was run.
