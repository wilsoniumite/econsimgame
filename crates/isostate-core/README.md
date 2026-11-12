# isostate-core

The simulation of Isolated State, in Rust: a library that the game links to,
and the `isostate-core` command. It began as a line-by-line port of the Python
package at the top of the repository, which stays as the reference, and has
grown the game's own rules beside the port, each behind a switch of its own.
There is no Bevy here and nothing that draws.

## Running it

Always name the crate, since the workspace also holds the viewer and the
generators:

```bash
cargo build -p isostate-core --release
cargo run -p isostate-core --release -- --help
cargo run -p isostate-core --release -- run --seed 7 --out runs/port7                             # the port of the Python
cargo run -p isostate-core --release -- run --seed 7 --rules game --families 6 --out runs/game7  # what a game is played under
cargo run -p isostate-core --release -- sweep --seeds 1-8 --rules game --families 6 --out runs/sweep-game
cargo test -p isostate-core --release    # 210 tests, four to six minutes once built
```

A run to 1900 takes 15 to 20 seconds on a quiet machine and writes `run.bin`,
`series_monthly.csv`, `events.json` and `params.json`. A sweep runs many
islands at once and boils each down to its stylised facts. Both are described,
with what they print, in the README at the top of the repository ("Other things
to run") and in [docs/rules/README.md](../../docs/rules/README.md).

The examples are small programs that use the library: `bots` plays the
player's family by four scripted strategies, `live` runs a game through the
live API and says how much the families own, `accounts` prints the books of
account over eight islands, `timing` times a run by its parts, and the rest are
censuses and probes kept from the work they were written for. Each says at its
top what it does and how to run it, as in
`cargo run -p isostate-core --release --example bots -- 1 2 3 7`.

## Where things are explained

- [PORTING.md](PORTING.md): how the Python was ported, the module for each of
  its modules, how every step was checked against it, every deliberate
  difference, and ten things in the Python that look like bugs.
- [FAMILIES.md](FAMILIES.md): the families, the API the game calls between
  years, and what the scripted players found.
- [docs/rules/README.md](../../docs/rules/README.md): the two sets of rules,
  the convention every new mechanism follows, the golden test that holds the
  port byte for byte (`tests/golden.rs`), the sweep and its facts; and beside
  it a page for each mechanism of the game's rules.
- `src/lib.rs` re-exports what a caller needs; each module says at its top
  what it does and, where it is not in the Python, which page explains it.
