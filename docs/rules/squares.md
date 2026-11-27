# Squares: where in its cell each built thing stands

**On under the game's rules since 26 November 2026; off under the port's. A shadow: kept from 1750, read by no rule, so the economy, the books and the news are the same to the bit with it on or off.** It is stage 6 of leaving the grid (the judge's design of 26 November, `study-offgrid`), in two pieces: the first keeps the squares, the second writes them into the record and hands them to a live game, as a log of what changed (below, "In the record").

The code is `squares.rs`, with two hooks in `sim.rs` marked "squares": `squares::open` as the run opens (in `start_from`, after the families) and `squares::close_year` at each year's end (in `end_year`, after every rule that changes the books). The state is `State::sq`. The record keeps the log in `record.rs` (`squares_snapshot`, `squares_log`), the live year in `live.rs` (`LiveYear::squares`), and the viewer reads it in its `replay.rs` (`Squares`). Not in the Python.

## Why

The books know that a cell of 2 km is so much works, so much pit and so much town, and not where in the cell each stands; the game's scenery invents that, and a replay and a live game can disagree. The designer asked on 26 November for the ground to be turned the other way round: squares small enough that a works may cover several, and no square split between a works and houses; and what is built should not jump about, since rebuilding has a real cost. The judge settled on squares of 125 m for built ground only (towns, works and pits), the open country keeping its hidden cells of 2 km. This piece puts the place of every built thing in the simulation, from 1750, and nothing reads it yet: later stages turn the picture and then the rules onto it.

## What it keeps

In every cell with something built, a table of sixteen by sixteen squares, each 125 m across (1.56 ha). Square `k = qy * 16 + qx` of cell c has its middle at (`c % W + (qx + 0.5) / 16`, `c / W + (qy + 0.5) / 16`) in the map's cells; the island is 1,920 squares across. A square has one tenant (`squares::Tenant`) and the year it was taken:

| tenant | what it is | how many squares |
|---|---|---|
| a works (which one) | its yard and buildings | its ground in the books, `works::land_held` (the capacity it has and has on order, by its hands at its method's square metres a hand: [works.md](works.md), part I), times 256; with the ground by method it keeps what it holds while that ground stays at `squares.keep` (0.7) of it or more. None for a shop, an office, care or a builder on a town's cell: those let floor in the house blocks (below, "Floor in the blocks") |
| a pit (which working) | the surface of a working | `mine_surface`, times 256 |
| house blocks (of a town) | houses, with their shops and offices | the record's `fill` (`ground::built_share`) of the cell's dry town ground, and never fewer than the floor let in them fills |
| a park (of a town) | | `ground::park_of` of the dry town ground (cells filled whole after 1900) |
| a town's open ground | yards, gardens, paddocks, lanes not yet built | the rest of a town cell |
| derelict ground (which works left it) | | the squares of a works that closed on a cell the books call derelict (`derelict_timer`), for as long as they do |

**Floor in the blocks** (`works.in_blocks`, [works.md](works.md) part J, since 26 November 2026). The works of a street, the shops and trades, the emporium, the offices, care and the builders, take no square of their own on a town's cells: their ground is floor let in the cell's house blocks. Each house block keeps how much of its floor is let and to what (`Table::floor`, one byte: the use in the high four bits as `squares::FLOOR_USES` numbers them, 1 shops, 2 offices, 3 care, 4 the builders, and the eighths of the block's floor in that use in the low four, 0 to 8). A block is let to one use. Each use on a cell has its works' ground in the books in eighths of a square (`works::land_held` times 2,048), and keeps what it holds while that stays at `squares.keep` of it, as a works keeps its squares; it takes the best blocks for a shop (beside the way, toward the town's middle), the offices, care and the builders whole blocks first, then the shops two eighths of a block (its street floor) on every block left, and more where that is not enough; a use other than the shops may take a block whose street floor alone is the shops', and the shops let theirs again in their turn. Where a cell's floor fills more blocks than its age has built, it has that many blocks while its open ground holds them. Off a town's cells such a works has a yard, as any other. So a town's middle is blocks, some of them let whole to offices, with shop fronts along the ways about them.

A way takes no square: it is a line over them. A cell with nothing built keeps no table. Beside each square the table keeps whether it lies under the sea or a lake (from the land's nodes, 500 m apart, taken between them), and which side the way lies on (below).

The squares of a cell come to its shares in the books within one square: the works' squares to `cell_K_share`, each works' to its own ground (its whole squares, and the cell's squares left over to the largest remainders); the pits' to the cell's surface; on a town cell, the town's ground to the cell less its works and pits, whole, as the books take it (`land.rs`, "taken"). A works under half a square's ground (a smithy of a few men) may hold no square of its own: it stands in a house block. With the ground by method (`works.by_method`, [works.md](works.md) part I) a works keeps the squares it holds while its ground in the books stays at seven tenths of them or more, as a building stands while some of its hands are laid off; below that it gives back down to its ground, and where its cell has no room for what is kept, the works keeping most over its ground gives way first (`squares::keep_squares`). The check then allows a works more squares than its ground, as far as that.

**The houses and the park are reckoned on the dry ground.** The books count every land cell as 4 km² of land, whatever the sea or a lake takes of it, and a town takes its cell whole. The town's squares are the cell's less its works and pits, as the books say, but its house blocks and park are the `fill` and the park's share of those squares that are dry, so that a town on the shore does not build in the sea. Its open ground takes the rest, the water among it.

## Placed once

The squares are allotted once a year, at the year's end, from what the books then say, and once as the run opens; between, nothing moves. Allotting is a reckoning of differences, never a laying out afresh:

1. What is given back first: a works or a working that holds more squares than its ground comes to gives back the squares it took last, so what it keeps is what it held at an earlier year, and is in one piece if that was. A works that closes gives back all of them: derelict ground where the books call the cell derelict, the open country otherwise.
2. Then what is taken: works first, those that take most first (so that a large works finds its ground in one piece before small ones scatter over it), then the workings. A works that holds nothing takes a first square; one that holds some takes the best free square beside its own.
3. Then the town's ground: every square of a town cell that no works or pit holds is the town's; house blocks and park that are over their count go back to open ground, the newest first; those under it take the best of the open ground.

So a tenant keeps its squares and their year for as long as the books leave it the ground, and what follows a works that closed takes its squares with a new year. A works never changes cell, and never moves within it. The years of the start are the books' own: a works of 1749 is dated by its founding, a first town's squares by the year its cell was built (`town_built`, before 1750), a working by its opening.

**A works hemmed in.** A works grows only onto free squares beside its own; town ground and derelict ground are free to it (house blocks at a cost that sends it round them where it can, the park at less), other works and pits are not. Where every square beside it is held by another works or a pit, it takes an annex: the free square nearest what it holds, as a works that cannot grow where it stands takes a yard over the way. Its holding counts its annexes (`Holding::annexes`), and only such a holding may be in more than one piece. They are not rare on crowded cells after 1850 (below).

## Where in the cell

A new square goes where its tenant does best by a score of the square, and the best wins; a hash of the square's place breaks ties and roughens edges. No draw is taken from the run's streams of chance. The terms, with distances in squares:

- **Beside the way that serves it.** The cell's ways are those the viewer draws: from its meeting place (`crossings`, where its roads meet) halfway to the meeting place of each neighbour, where both cells have a road; a cell with none has its meeting place as the point a way would come to. Every tenant but the park is drawn to the way: house blocks and shops most (0.25 and 0.3 a square), works less (0.2), pits least (0.05).
- **Toward the town's middle, or away.** The middle is the meeting place of the market town's first cell. House blocks lean toward it (0.12 a square), and so do shops and offices (the works that make services, 0.15); every other works and every pit leans away (0.06 a square, up to 24 squares), so that the yards go to the town's edge.
- **Beside water for a mill.** A works whose method is a mill (its name says so: the powered mill, the rolling mill, the rail mill) is drawn to the nearest water (0.35 a square, up to 16): a river's line, a lake or the sea.
- **Off steep and wet ground.** Each square's grade is taken from the land's heights between the nodes 500 m apart (the same smooth surface the meeting places are found on, `crossings::metres_at`), over a square's width; its wetness is the node's (dry, flood plain, marsh). The cost is 1 at a grade of 1 in 12 and rises as its square, to at most 25, with 0.6 more on the flood plain and 2.5 in the marsh. House blocks weigh it half as much again; pits half. A square under water is taken by works or houses only when nothing else is left.
- **A park takes the poor ground**: the wet, the steep and the waterside.
- **A works' first square** is set in the smallest piece of the cell's free ground that holds what it needs, so that the large pieces are kept for large works, and as far from other works and pits as its size asks (up to the square root of its squares plus one), so that it has room to grow. A growing works prefers squares with more of its own beside them, and fewer of other works'.

## Square to the way, not to the map

The squares lie on the map's axes, and a later stage that draws a yard square by square would show a lattice of 125 m. Each square keeps, from the moment it is taken, which side its way lies on (`Table::face`: the bearing from the square's middle to the nearest point of the cell's ways, in 256ths of a turn, 0 toward the map's east and 64 toward its south). Each works and working keeps the same for its first square (`Holding::face`) and the heading of the way there (`Holding::along`, in 256ths of a half turn), which is what a yard should be turned square to. What else would help a later stage: the way's own course and level at that point once roads are kept by link (the judge's stage 1 draws only the links carts wore), and for a town cell the line of its high street, so that its blocks can be laid out from that and not from the eight half-links of the cell, which make stars (below).

## In the record

The squares reach the record and a live game as a log, not as a picture of every year: each year's end, the changes it made.

**One list, two readers.** At the end of `close_year` the book is compared, square by square, with what the log last told of it (every table, those given up too), and every square whose tenant, year or face is not what was told goes into `SquareBook::changes`, in order of the square (`Change::at`, the cell times 256 plus the square). A change says what the square now is: its tenant's code, the year the tenant took it, its face and its floor let (`Change::floor`, 0 but on a house block with floor let); a square given up is told as tenant, year, face and floor 0. A block whose floor let changes is a change, its tenant and year the same. The first year's changes are made out of no squares at all, and carry the opening's. The record copies the list into its log for the year (`record::squares_snapshot`), and the live year hands the same list over (`LiveYear::squares`), so a live game and a replay of the same run hold the same squares by construction. A live cursor counts the year's ends it has told; one that has missed a year is sent the whole book, marked `whole`, and a reader starts again from it.

**In `run.bin`.** Two arrays, written only by a run that keeps the squares: `sq_log`, the bytes of every year's changes one year after another, and `sq_first`, where each year's begin (n_years + 1 of them, as `mine_first`). The header says `"squares": {"layout": 2, "side": 16, "metres": 125, "kinds": [...], "floor": [...]}`. A year's bytes (`squares::encode_year`, read by `squares::decode_year`) go cell by cell in rising order: the cell less the one before it, the number of runs, and each run of squares given to one tenant with one year and one floor: a head, `((number << 3 | kind) << 1 | dated) << 1 | floored` with the kind as `squares::KINDS` numbers it (0 for squares given up), the year less the log's own year where `dated` says it differs, zigzagged, the floor's byte where `floored` says it is let, the count, and for each square its place in the cell and its face (no face for a square given up). Layout 1, before the floor, had no `floored` bit; a reader of layout 2 refuses a record of layout 1 by its header's word, as the viewer does, and says so. Every number but the squares and faces is a varint, seven bits a byte, low first. A square taken by a tenant that takes others in its cell that year is two bytes.

**Old records and saves.** A record made before the squares were kept, or under the port's rules, has neither array nor the header's word: it opens as it always did, and the viewer says in its log that the run "keeps no squares of 125 m". A save is a log of orders, played again under today's rules, so its squares are worked out again as it plays and come out as they were (`tests/squares.rs`, `a_save_plays_again_with_its_squares_and_a_port_record_keeps_none`); a save made before the squares were kept gets them the same way.

**The viewer** reads the log into `Builds::squares` (`replay.rs`, `Squares`) and draws nothing from it yet. It can give the squares of any cell in any year (`Squares::cell`, a binary search in each year's list) and every cell's in a year (`Squares::year`, the log read once to that year); both read the log and nothing else, so a scenery build can ask for them off the frame. A record's years are kept as their bytes and read the first time they are asked for, so that opening a record does not wait for them.

**What it measured.** Islands 7 and 2, game rules, six families, to 2050, with a live cursor beside the run as a game has (`cargo run --profile work -p isostate-core --example squares_log -- --seeds 7,2 --to 2050`):

| | island 7 | island 2 |
|---|---|---|
| years in which the live game held the book's squares | 301 of 301 | 301 of 301 |
| years in which the record, packed and read back, held them | 301 of 301 | 301 of 301 |
| squares held in 2050 | 240,171 | 253,521 |
| changes in all, 1750 to 2050 | 1,952,146 | 1,962,742 |
| the first year's, with the opening | 22,617 | 23,290 |
| the most in a later year | 18,644 (2039) | 19,710 (2041) |
| the log | 4.37 MB, 2.24 bytes a change | 4.52 MB, 2.30 bytes a change |
| `run.bin` with the log, and without | 260.6 and 256.2 MB (+1.7%) | 258.0 and 253.4 MB (+1.8%) |
| packing the record, with the log and without (median of 3) | 559 and 432 ms | 524 and 457 ms |
| of it, writing the log alone; reading it all back | 71 ms; 38 ms | 72 ms; 40 ms |

The viewer opens island 7's record of 2050 in 396 ms with the log and 392 ms without it (+1%, medians of seven openings of each in turn, wall clock, the dev profile; `ISOSTATE_TIME_OPEN="a.bin;b.bin" cargo test -p isostate-viewer -- --ignored time_to_open --nocapture`); reading every year's squares the first time, to 2050, then takes 103 ms. Read whole at opening instead, the log made the opening 13 per cent slower (445 ms against 395), which is why it is left until it is asked for.

Most of the changes are works' ground going up and down by a square from one year to the next (the books' capacity on order comes and goes): of island 7's 1.95 million, 630,000 give a square to a works and 454,000 take one from a works to nothing, 207,000 take house blocks for works and 199,000 give them back; 310,000 give a square back to the tenant it had in the three years before. They fall where the works are: on island 7 the busiest cells, in the five large towns, change 13 to 14 squares a year on the mean.

## For the stages that draw from them

What the builders of the judge's stages 8 (works on their squares) and 10 (the viewer's towns after 1900) will find, and what they will not, from the pictures of the shadow and the viewer's towns as they are:

- **In the log:** each square's tenant (which works, which town's house blocks, park or open ground, which working's pit, which works left derelict ground), the year it was taken, and its face, the side its way lay on when it was taken. A works' squares are those with its tenant in its cell; never in two cells, today.
- **Not in the log, kept in the core:** a holding's first square and the order its squares were taken (`Holding::squares`), which is where it first stood and which way it grew; `Holding::along`, the heading of the way at its first square, which is what a yard should be turned square to; its annexes. A yard laid from the log alone can take its heading only from the faces (square to the most common face of its squares), which is near enough beside a straight way and wrong at a corner. Stage 8 should put `face` and `along` of each holding into the record (a few bytes a works when it first takes ground) before it lays yards.
- **The face points to the core's ways, not the drawn ones.** The core reckons a square's way as the viewer joined its roads before 26 November: from the cell's meeting place half way to each of its eight neighbours' wherever both cells have any road. The viewer now draws only the links carts wore (the judge's stage 1, `network::drawn_links`), so a face may point to a way that is not drawn. Stage 8 should either reckon faces again against the drawn links, in the viewer, or have the core keep the links the carts wore.
- **Height and slope:** not in the log. The viewer has the same land (the 500 m nodes, `newland`), so it can take a square's height, slope and whether it is under water at its middle itself, as the core does with `crossings::metres_at`. Under water matters: on island 7 about 1,700 squares of works and pits lie under the sea or a lake in 2050, which a picture must place on piers or leave out.
- **Neighbours:** a square's neighbours in its cell are in the same year's table; across a cell's edge they are in the next cell's (`Squares::year` gives every cell at once). A works never reaches over its cell's edge, so the cells' lattice shows in the works' outlines until the ground a works takes (stage 7) lets it.
- **Stars and slabs.** House blocks are placed toward the eight half-links, so a cell part built grows along a star; and a works takes 2,000 m² a book hand, so the middle of a large town is a slab of works a cell wide by 1950. A town drawn square by square from the log (stage 10) will show both until the ground of a works (stage 7) and a frame from the high street are mended. Drawn now, the squares would make the picture worse, not better.
- **No density by the square:** how densely a block is built is a cell's (the record's `dens` and `fill`); every house square of a cell is built alike. An estate as a run of squares of one year (stage 10) can be read from the log's years.
- **No year of rebuilding:** the year is the year taken. Stage 11's "no look changes without a rebuilt year" needs a year rebuilt by the square, which nothing keeps yet.
- **Reading in a scenery build:** `Squares::year` reads the log from the start each time it is called (about 100 ms to 2050 in the dev profile). A build that goes year by year should keep the tables it made and lay each year's changes on them (`Squares::changes`), a few milliseconds a year.

## Its numbers

`SquaresParams`, in `squares.rs`. Off under `default_params()`; `SquaresParams::game()` in `game_params()`.

| | port | game | |
|---|---|---|---|
| `enabled` | false | true | the simulation keeps the squares |

The weights of the placing rule are in the code, at `squares::score`: they place, and move nothing in the economy.

## What it measured

Island 7, game rules, six families, to 2050 (`cargo run --profile work -p isostate-core --example squares -- --seeds 7 --to 2050 --check`). Squares held, by tenant:

| year | cells with a table | works | house blocks | park | pits | derelict | town's open ground | built (all but open) | works alive | of them with no square | of them in pieces |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1750 | 257 | 2,636 | 17,391 | 0 | 46 | 0 | 2,544 | 20,073 | 351 | 26 | 0 |
| 1800 | 330 | 6,643 | 25,870 | 0 | 262 | 0 | 6,980 | 32,775 | 455 | 19 | 21 |
| 1850 | 515 | 19,097 | 35,196 | 0 | 725 | 0 | 9,632 | 55,018 | 910 | 68 | 108 |
| 1900 | 979 | 43,923 | 69,318 | 0 | 1,158 | 98 | 21,215 | 114,497 | 1,364 | 60 | 222 |
| 1950 | 1,099 | 112,676 | 72,436 | 1,962 | 726 | 0 | 5,185 | 187,800 | 2,303 | 97 | 391 |
| 2000 | 1,114 | 143,442 | 52,937 | 1,909 | 706 | 881 | 3,018 | 199,875 | 2,148 | 113 | 276 |
| 2050 | 1,180 | 183,849 | 50,029 | 1,899 | 962 | 679 | 2,753 | 237,418 | 2,340 | 60 | 292 |

The judge reckoned about 260,000 built squares by 2050, 186,000 of them works and at most 78,000 house blocks: 237,418, of them 183,849 works and 50,029 house blocks. The house blocks were most in 1930 (83,101) and have given ground to the works since.

Every year of that run, and of islands 1 to 8 to 2050 (`--seeds 1-8 --check`), `squares::check` held: each works' squares its ground within a square, each cell's works and pits theirs, a town cell's town ground the cell less its works and pits, its house blocks and park their counts, no square with two tenants, no holding in more pieces than its annexes allow, no derelict ground off a derelict cell, and every square that kept its tenant kept its year. With the switch on and off the game's golden hashes are the same, all seven (`tests/golden_game.rs`), and so are the months, the news and the record of island 5 to 1790 (`tests/squares.rs`).

Those are the numbers before the works of a street went into the blocks. With them (`works.in_blocks`, [works.md](works.md) part J; `examples/works_blocks.rs --seeds 1-8`), on the mean of islands 1 to 8 no works' square on a town's cells is a shop's, an office's, care's or a builder's in any year, and the house blocks with floor let are 8,159 in 1900, 2,214 in 1950, 656 in 2000 and 738 in 2050; the works' squares in 2050 are 60,574 for 68,285 house blocks. The check (`examples/squares.rs --seeds 7,2,4,8 --to 2050 --check`) holds every year on the four islands; it allows a cell one block of houses more than its floor let fills, where floor is let (the blocks are counted from the floor wanted, which can be an eighth or two more than the floor let where the shops' street floor is spread thin: 8 years of 1,204 before it was allowed).

On islands 1 to 8 in 2050 the built squares are 224,000 to 280,000 (252,000 on the mean), the works' 175,000 to 215,000 and the house blocks' 46,000 to 65,000; a tenth to a sixth of the works have an annex.

**What it costs.** The year's allotting, timed alone beside a run of island 7 to 2050 (`--time`, the squares switched off in the run and `squares::close_year` called after each year on the same state, on a machine shared with other builds): 2.0 s in all, 1.5 per cent of the run; a year's median 6.3 ms (4.0 ms to 1850, 8.2 ms from 1851 to 1950, 6.0 ms after), the 99th of 301 years 12.6 ms, the slowest 17.9 ms (2039); the opening, once, 52 ms. A sweep of islands 1 to 8 to 2050 with the switch off and on side by side (`isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set squares.enabled=false,true`) took 108 and 109 processor seconds a run on the mean, and gave every one of its facts the same on every island.

The book of 2050 holds about 1,200 tables in use, of about 4.4 kB each with the ground worked out under them (a table given up is kept, empty, for its cell), and the works' and workings' lists: some megabytes.

## What worries me

- **The works' ground.** The squares draw the books' ground, and the books gave a works 2,000 m² a hand of capacity in every trade and year (`works::land_take`). By 1950 the works held most of the largest town's middle, a cell each as a slab of squares, and the houses were pushed to the edge cells; on island 7 the works' squares passed the houses' about 1930 and were 3.7 times them by 2050. The ground by method ([works.md](works.md), part I, 26 November 2026) brings islands 1 to 8 to 0.95 of the houses' squares in 2050 for 3.04 (island 7 1.05); what is left is the old methods' 2,000 m² a hand of plant, which waits for the old towns to be laid out anew.
- **Works in pieces.** A works hemmed in by other works and pits takes an annex. By 1790 that is 3 to 8 per cent of the works on islands 1 to 8, and on island 7 by 2050 292 of 2,340 (12 per cent), nearly all on cells the works hold nine tenths of. Without moving a neighbour no rule can avoid it once a cell is that full; the mend is the ground rule above, or letting a works cross its cell's edge (it never does here, since the books keep its ground on its cell, and so the cells' lattice still shows in the works' outlines).
- **Stars.** A cell's ways are the viewer's half-links from its meeting place to each of its eight neighbours', so in a cell part built the house blocks gather along an X or a star (the pictures of 1800 and 1850). A cell whose roads are kept by link, or a high street, would give a truer frame.
- **The park comes in specks.** The park takes the poorest squares of each cell filled whole, which are the wet and steep ones scattered about it: green specks, not a park. A park should be one piece, grown as a works is.
- **Water.** The books count a coastal cell whole. Houses and parks are kept on the dry ground, but works are not: 1,725 squares of works and pits on island 7 lie under the sea or a lake in 2050, where a cell's works hold more than its dry ground. The town's open ground takes the rest of the water.
- **An old cell is solid.** A town cell built long ago is filled toward the ceiling, so its house blocks take nearly all of its dry ground and leave no open ground: a small old town (Maredge, island 7) is one solid block of 256 squares. Streets are lines over squares and take none, but a later picture will want lanes, yards and gardens inside a block.
- **The log flickered.** A sixth of the log was a square going back to the tenant it had a year or three before, as a works' ground in the books rose and fell by a square: on island 7 to 2050, 303,033 of 1.88 million changes. With the ground by method a works keeps its squares while its ground stays at `squares.keep` (0.7) of them, and 25,942 of 790,582 changes do (`examples/works_ground.rs`; islands 1 to 8, 22,384 to 27,508 against 289,141 to 351,150). The price is squares held over the books' ground: in 2050 the works' squares are 12 per cent over their ground in the books (61,981 squares, 968 km², for 861 km²).
- **Once a year.** A works founded in the year has its squares at the year's end, not on its founding day; the books share land out once a year too, so nothing reads the difference.
