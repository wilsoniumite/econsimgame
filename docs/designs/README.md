# Designs

Designs that have been worked out and argued over but not yet built, or only
partly. Each says at its head what it is for and where it stands.

- `production-tree.md`: the game's deeper production tree for 1750 to 1900, in
  stages, with how each chain is expected to carry on toward 2050. Made on
  11 November 2026 from three independent designs (history first, map first,
  engine first), then checked by a critic and revised. `production-tree-draft.toml`
  is the overlay it proposes over `data/content.toml`, every number filled in;
  `production-tree-critique.md` is the critic's report with the unit sums.
  It is a reckoning at fixed prices, not yet a run: the numbers will move when
  the stages are built and swept.
- `architecture.md`: a review of the code's structure, made on 12 November 2026
  from five surveys of the three crates: what is in good shape and should be
  left alone, what slows or endangers the work, a verdict on four guesses made
  beforehand, and nine small changes in the order to make them (before the rules
  pass, beside the new island, after both), with those not worth making. Its
  answer is that no large refactor is due. It changes no code.
