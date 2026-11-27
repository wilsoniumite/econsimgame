# Courses: which ways the carts wore, where they pass and the course they run

**On under the game's rules since 27 November 2026; off under the port's. Read by no rule: the economy, the books and the news are the same to the bit with it on or off.** What moves with it is where the squares stand ([squares.md](squares.md)), which now face the kept courses, and the record, which carries a log of the ways and of the order each works took its squares in.

The code is `courses.rs`, with two hooks in `sim.rs` marked "courses": `courses::open` as the run opens (in `start_from`, just before `squares::open`) and `courses::close_year` at each year's end (in `end_year`, just before `squares::close_year`). The state is `State::wc`. The record keeps the log in `record.rs` (`courses_snapshot`; the arrays `way_first` and `way_log`), the live year in `live.rs` (`LiveYear::ways`). Not in the Python.

## Why

The designer asked on 26 November to leave the grid: "I hope that might actually improve the look a lot, especially for the current grid of roads", and then for small squares of one use that a works may span. The two had come apart. The simulation's squares of 125 m faced eight straight half-links from the middle of each cell, so a town's houses ran out in stars; the game drew a road only where carts had worn one, on a course it found over the ground for itself. When works and houses are drawn on their own squares, they would stand beside roads that are not there. One thing must know where a way runs, and it has to be the simulation, because the squares are its own.

## What it keeps

**Which ways exist.** A link is a cell's way to one of its eight neighbours, kept on the square that holds it as the traffic record has it (`traffic::SLOTS`: east, south, south-east, south-west). Since 26 November 2026 the game has drawn a road along a link only where the carts wore one (the viewer's `network::drawn_links`). The same rule now runs in the simulation, on the same numbers, at the same moment of the year (after the year's carting, before the record's snapshot):

- a link between two squares of made road (level 2 or 3 at both ends) is **worn** when it has carried 1,500 or more by cart in 10 of the last 20 years, a year of 40,000 or more counting as five; in the record's first ten years, when it has done so in every year so far;
- a worn link goes back to grass after 20 years in which it never carried 1,500;
- a link is **drawn** while it is worn, or while it is a trunk road's or a bypass's and today's rule joins it (`road_segments`: every two neighbouring squares of made road, a diagonal left out where a pair of straight links joins the two);
- the weight is read as the record's byte keeps it (`traffic::goods_byte`, then `byte_goods`), so that the simulation and the viewer read the same number.

The year each link was first drawn is kept (`CourseBook::first`).

**A cell's place.** The ways through a cell pass at one place (`CourseBook::place`), chosen the year the first way reaches the cell and never moved after. A cell that is a town's when its first way comes keeps its meeting place (`crossings::meeting_place`), where the game draws its market and lays its streets, so that no street of a town moves. Elsewhere **the ground chooses**:

- the spots tried are the meeting place and a grid of nine by nine over the cell, kept 300 m from its edges;
- a spot in water, on a river's band (its half width and 100 m), or on ground under 1.8 m above the tide is not taken; nor one steeper than 1 in 8;
- of the rest, the place is the spot where **the ways that meet there that year run cheapest** to the places at their other ends (or their meeting places, where those have no place yet), each priced as a lane priced its straight way over the ground (below), with a little for steepness short of 1 in 8, and half a cell of way for each cell of distance from the meeting place. That last term keeps the place of a road's end from sliding to the edge of its cell; it is the only lean.

**A link's course.** Found once, the year the link is first drawn, over the ground between its two places (`seek`), and kept for ever, in the state and in the record, as a line of points in 256ths of a cell (7.8 m). The search is the viewer's (`road_ground::seek`) over the simulation's own ground:

- squares of 125 m in a box 800 m round the two places, sixteen steps from each (the eight neighbours and the knight's moves);
- each step costs its length (a cell of 2 km is 1), and for each metre it climbs or falls the metres of level road its kind says a metre is worth, and far more for each metre beyond its ruling gradient: a lane 20 and 60 beyond 1 in 12, a turnpike (level 3) 20 and 60 beyond 1 in 20, a trunk road 10 and 30 beyond 1 in 25, as the viewer prices them;
- wet ground, for each cell of length: standing water 40, ground under 1.8 m 3, marsh 1.5, a valley's floor 0.3 (a trunk road half); a town's ground that is neither of the link's own cells 6; a river's band 16, so that a way crosses a river square to it and once; and for straying from the straight line between the places, 0.5 a lane, 1 a turnpike, 3 a trunk road, for each cell off it for each cell of length;
- the way found over the squares is pulled taut: from each point, straight to the furthest point the straight line reaches for no more than the squares' way costs between them, with 3 per cent of slack. What is kept is the taut line's corners, the two places first and last.

The ground is the simulation's: heights from `crossings::metres_at`, the smooth surface through the land's nodes 500 m apart that the game lays its own heights through (the viewer's `newland/ground.rs`), water and lakes from the nodes, marsh and flood plain from the nodes' wetness, and the rivers' lines from the land, those the game draws (a cubic metre a second or more) above the tide. No height the game draws moves.

**When a road rises a level.** A course is found for the kind of road its link is the year it is first drawn, and kept: a lane made a turnpike, or a trunk road, keeps its course. That is what the game draws today (its own courses are found once and kept, `road_ground.rs`); a road that went back to grass and is worn again comes back on its old course. Finding a straighter course for a turnpike would move a line already drawn, and every house and works that has stood beside it; a later piece may let a trunk road take a new course of its own beside the old lane, which then stays.

**The roads that stand in 1750.** The island's roads are older than the record. As the run opens, every made road that today's rule joins is given its places and its course, dated the year before the first; the squares of the opening face them, so that the towns of 1750 line their own roads. From the first year on, links are drawn by the carts.

## The squares read it

With the courses on, a square's way (`squares::frame`) is no longer the cell's eight half-links: it is every kept course that comes within half a cell of the cell, and the cell's place where no course does. A course whose link the carts no longer wear still counts: a town draws such a road as its own street. Each square keeps, as before, the bearing from its middle to the nearest point of a way when it is taken (`Table::face`), and a works or working the heading of the way at its first square (`Holding::along`).

**Frontage first.** A house block or a shop's square within one square (125 m) of a course is worth one more in the placing score (`squares::FRONTAGE`, about eight squares nearer the town's middle), so a town takes its frontage along a way before the ground behind.

**Nothing is moved.** A square once taken keeps its tenant and year until the books take its tenant's ground away, as before: a way found later changes where the next squares go, never the squares already taken. Switched off, the book is empty and the squares face the half-links as they did.

## In the record

**The ways' log** (`way_first`, `way_log`, the header's `"ways": {"layout": 1, "unit": 256, "slots": [...], "kinds": ["lane", "turnpike", "trunk"]}`), written only by a run that keeps the courses. Year k's bytes are `way_log[way_first[k] .. way_first[k + 1]]` (`courses::encode_year`, read back by `courses::decode_year`):

- the number of cells placed; for each, the cell less the one before it, its place as two little-endian `u16`s in 256ths of a cell, and the year it was chosen, zigzagged;
- the number of links; for each, the link (`cell * 4 + slot`) less the one before it, a byte (1: drawn from this year on; 2: its course follows), and where its course follows, the year it was found (zigzagged), its kind, its number of points and each point as a zigzagged step from the one before. A course's first point is the place of the link's own cell, its last the place of its neighbour.

Every other number is a varint, seven bits a byte, low first. A year lists the places chosen and the courses found since the last, and every link drawn or no longer drawn. The first year's log is made out of nothing and carries the opening's roads too. A reader that applies every year in turn holds each cell's place, each link's course and the links drawn that year, as the simulation does (`tests/courses.rs`).

**The holdings' log** (layout 3 of the squares: `sq_hold_first`, `sq_hold`; `squares::encode_holds`, `decode_holds`). For each works and working that took squares in a year: its cell, its number (a working's by its resource row), whether it held none before, and then the heading of the way at its first square (`Holding::along`, in 256ths of a half turn), and the squares it took, in the order it took them. A holding's squares are a stack: the first it ever took is where it first stood, and what it gives back is always what it took last, which the squares' log shows going. The first year's holdings' log is every holding's squares from nothing. `sq_log` is laid out as in layout 2, so a reader of layout 2 reads it (the viewer reads both, `squares::LOG_READS`); a record written before has no `sq_hold` and reads as it did.

**A live game** is handed the same bytes each year (`LiveYear::ways`, `LiveSquares::holds`); a cursor that has missed a year's end is sent the whole book (`courses::whole`, `squares::whole_holds`), marked `whole`.

## Its numbers

`CourseParams`, in `courses.rs`. Off under `default_params()`; `CourseParams::game()` in `game_params()`. It acts only where the traffic is kept (`traffic.enabled`, whose carting wears the ways) and on the new land (whose heights and rivers the courses read).

| | port | game | |
|---|---|---|---|
| `enabled` | false | true | the simulation keeps the ways, their places and courses, and the squares face them |

The wear's numbers are the viewer's (`courses::WORN_*`, which must stay equal to `network::WORN_*`; the viewer's test `the_roads_drawn_are_those_the_carts_wore_and_close_few_squares` holds the two to the link). The prices are in the code, at `courses::price` and the constants beside it.

**A works wants its way within reach.** With the eight half-links every square of a cell was near a way; with the courses a cell has one or two, and works pulled to them as before (0.2 of a point a square) crowded along them and hemmed each other in: works in more than one piece in 1790, islands 1 to 8, rose from 103 to 155. So with the courses a works' yard counts its way as reached within three squares (`squares::YARD_REACH`, 375 m); house blocks and shops still take the frontage. In pieces then: 131.

## What it measured

Island 7 under the game's rules with six families, played to 2050 twice side by side, with the courses on and off (`cargo run --profile work -p isostate-core --example courses -- --seeds 7 --to 2050 --time`, and `--start landfall`). The economy, the books and the news were the same in every year on both starts; every place stayed where it was chosen and every course kept its points; played as a live game, every year's ways' log, squares' log and holdings' log handed over were the record's bytes, and a reader of the ways' log held the simulation's places, courses and drawn links.

| island 7, settled | 1750 | 1800 | 1850 | 1900 | 1950 | 2000 | 2050 |
|---|---|---|---|---|---|---|---|
| links drawn | 878 | 1,034 | 1,812 | 2,061 | 3,259 | 2,819 | 2,292 |
| courses kept | 1,348 | 1,738 | 2,646 | 4,757 | 6,513 | 7,002 | 7,136 |
| cells placed | 1,026 | 1,327 | 2,013 | 3,268 | 3,789 | 3,885 | 3,969 |
| drawn courses' length within 5 degrees of the eight directions | 0.357 | 0.365 | 0.376 | 0.358 | 0.356 | 0.349 | 0.349 |
| the same, straight between meeting places | 0.496 | 0.470 | 0.483 | 0.458 | 0.486 | 0.478 | 0.476 |
| house blocks taken since 1750 within a square of a kept course, courses on | 0.184 | 0.245 | 0.226 | 0.256 | 0.286 | 0.283 | 0.294 |
| the same, courses off | 0.178 | 0.194 | 0.199 | 0.213 | 0.279 | 0.284 | 0.297 |
| works' first squares within a square of a kept course, on | 0.604 | 0.535 | 0.548 | 0.557 | 0.615 | 0.622 | 0.560 |
| the same, off | 0.185 | 0.240 | 0.263 | 0.255 | 0.309 | 0.287 | 0.275 |

On the landfall start (`courses-7-landfall`, before the works' reach): 75 links drawn in 1750, 2,071 in 1900 and 3,615 in 2050; within 5 degrees of the eight directions 0.341 to 0.426 (straight 0.47 to 0.61); house blocks near a course 0.206 against 0.192 in 1800 and 0.215 against 0.188 in 1900, and the same with or without by 2000.

What it says. The courses leave the lattice: a third of their length lies within 5 degrees of the map's eight directions, where the straight links between meeting places have nearly half, and the game's own courses over the ground 0.397 (island 7, 2050, roads-ground). Up to 1900 a fifth more of the new house blocks stand on a course's frontage, and the stars are gone from part-built cells (the pictures); after 1950 the kept courses are so many (7,000 on island 7, faded ones among them) that nearly every square is near one, and the share is the same either way. Twice as many works stand by their way. The squares of the opening are on ground built before 1750 and stand as the books dated them.

**What it costs.** At the year's end on island 7 to 2050, timed alone beside a run (one thread, the clock, a shared machine): the courses 1.29 s in all, a year's median 3.4 ms, the most 22 ms (1750, the opening's roads); the squares 2.54 s, median 7.8 ms, against main's squares 1.79 s, median 5.5 ms taken beside it (they now look at more ways). The year's end of the two together is about 11 ms on the median against 5.5. A whole run of island 7 to 2050, in processor seconds, main and the branch in turn: 115.8 and 129.6, then 113.0 and 114.4. The record of island 7 to 2050 is 256.8 MB against 256.0 (+0.29 per cent): the ways' log 191 kB and the holdings' log 547 kB. With the courses off the squares' log is main's to the byte.

## What worries me

- **Two rules of wear.** The viewer still decides what it draws by its own `network::drawn_links`, and the simulation by `courses::wear`, the same rule written twice. A test holds them to the link on island 7 for forty years; the next step (ways-read) should make the viewer read the simulation's.
- **The course is not the game's course.** The viewer's own finder (`road_ground.rs`) leaves each end along a direction fixed through its meeting place, starts from the meeting places and not from these places, and crosses a river only at the simulation's bridge; these courses do none of that, and cross a river where the ground makes it cheapest. Until the game draws these courses, a square's face points to a course that is not drawn.
- **Bridges.** The crossings' table (`crossings.rs`) prices a link's fords and bridges where the straight line between meeting places crosses a river; a course may cross it elsewhere, or a link whose straight line crosses none may have a course that does. Nothing in the economy reads the courses, so the books are not wrong; the picture would put the bridge where the course crosses.
- **Zigzags.** On a steep valley side a lane's course climbs in short hairpins, as the prices say a carter would; drawn as it is kept it looks nervous. The game should ease it when it draws it, and not keep the easing.
