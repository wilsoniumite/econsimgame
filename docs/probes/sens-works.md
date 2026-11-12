# How sensitive the island is to the works' life cycle and to carting

One of six tables of sensitivity for the pass over all the rules (piece 18 of
`docs/backlog/found-along-the-way.md`; the findings behind this one are R119 and C34, "the life
cycle's numbers were checked only as a set", and R13 and C92, "the plan's targets for 1750 to
1800 were set without carting"). It changes no rule and no fact. Every number of the two
mechanisms was moved on its own, to about three quarters and four thirds of the game's value,
and the island run again; this page says what moved.

The game's rules at a6ff0ec (12 November 2026), six families, to 1900. A first draft, from seeds
1 to 8; the sixteen-island check and the placebo follow.

## How it was run and read

Every setting is a field of `WorksParams` (`crates/isostate-core/src/works.rs`) or
`CartingParams` (`crates/isostate-core/src/carting.rs`), and `--set` reaches each of them:

```bash
isostate-core sweep --seeds 1-16 --rules game --families 6 --out DIR/base16
isostate-core sweep --seeds 1-8 --rules game --families 6 --set works.year_room=0.1125,0.2 --out DIR/s8-works.year_room --compare DIR/base16
```

and so on for each key. Each run is set beside the same island under the game's own values.
For every fact and for twenty-year means of the yearly series (1780-99, 1830-49 and 1880-99, from
`annual.csv`; one year misleads) the reading gives the mean change, the islands up and down, and
the change over the spread across islands, written *d*: the change in the mean divided by the
standard deviation of the fact across islands. A *d* of 1 means the setting moves the fact by as
much as one island differs from the next. Swings are the standard deviation of the year-on-year
change of the logarithm, 1800 to 1899.
