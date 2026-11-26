# Squares: where in its cell each built thing stands

**On under the game's rules since 26 November 2026; off under the port's. A shadow: kept from 1750, read by no rule, so the economy, the books and the news are the same to the bit with it on or off.** It is stage 6 of leaving the grid (the judge's design of 26 November, `study-offgrid`), the first of two pieces: this one keeps the squares, the next writes them into the record for the viewer.

The code is `squares.rs`, with two hooks in `sim.rs` marked "squares": `squares::open` as the run opens (in `start_from`, after the families) and `squares::close_year` at each year's end (in `end_year`, after every rule that changes the books). The state is `State::sq`. Not in the Python.

## Why

The books know that a cell of 2 km is so much works, so much pit and so much town, and not where in the cell each stands; the game's scenery invents that, and a replay and a live game can disagree. The designer asked on 26 November for the ground to be turned the other way round: squares small enough that a works may cover several, and no square split between a works and houses; and what is built should not jump about, since rebuilding has a real cost. The judge settled on squares of 125 m for built ground only (towns, works and pits), the open country keeping its hidden cells of 2 km. This piece puts the place of every built thing in the simulation, from 1750, and nothing reads it yet: later stages turn the picture and then the rules onto it.

## What it keeps

In every cell with something built, a table of sixteen by sixteen squares, each 125 m across (1.56 ha). Square `k = qy * 16 + qx` of cell c has its middle at (`c % W + (qx + 0.5) / 16`, `c / W + (qy + 0.5) / 16`) in the map's cells; the island is 1,920 squares across. A square has one tenant (`squares::Tenant`) and the year it was taken:

| tenant | what it is | how many squares |
|---|---|---|
| a works (which one) | its yard and buildings | its ground in the books, `works::land_held` (the capacity it has and has on order, by its hands at `hands_per_cell`), times 256 |
| a pit (which working) | the surface of a working | `mine_surface`, times 256 |
| house blocks (of a town) | houses, with their shops | the record's `fill` (`ground::built_share`) of the cell's dry town ground |
| a park (of a town) | | `ground::park_of` of the dry town ground (cells filled whole after 1900) |
| a town's open ground | yards, gardens, paddocks, lanes not yet built | the rest of a town cell |
| derelict ground (which works left it) | | the squares of a works that closed on a cell the books call derelict (`derelict_timer`), for as long as they do |

A way takes no square: it is a line over them. A cell with nothing built keeps no table. Beside each square the table keeps whether it lies under the sea or a lake (from the land's nodes, 500 m apart, taken between them), and which side the way lies on (below).

The squares of a cell come to its shares in the books within one square: the works' squares to `cell_K_share`, each works' to its own ground (its whole squares, and the cell's squares left over to the largest remainders); the pits' to the cell's surface; on a town cell, the town's ground to the cell less its works and pits, whole, as the books take it (`land.rs`, "taken"). A works under half a square's ground (a smithy of a few men) may hold no square of its own: it stands in a house block.

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

## Its numbers

`SquaresParams`, in `squares.rs`. Off under `default_params()`; `SquaresParams::game()` in `game_params()`.

| | port | game | |
|---|---|---|---|
| `enabled` | false | true | the simulation keeps the squares |

The weights of the placing rule are in the code, at `squares::score`: they place, and move nothing in the economy.

## What it measured

Island 7, game rules, six families, to 2050 (`cargo run --profile work -p isostate-core --example squares -- --seeds 7 --to 2050 --check`). Squares held, by tenant:

RESULTS_TABLE

The judge reckoned about 260,000 built squares by 2050, 186,000 of them works and at most 78,000 house blocks.

## What worries me

WORRIES
