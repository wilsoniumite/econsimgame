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
- `rules-pass.md`: the pass over all the game's rules together, made on
  12 November 2026 from the traces and tables of sensitivity of that day: a
  scorecard of twenty aims for 1750 to 1900, each with its tolerance and where
  it is stated; fourteen changes, from hands that exist to who counts as
  living in a town; and eleven stages to build them in, five before the
  production tree lands and six after, each read against a null pair of
  histories that differ only by chance. A critic ran four of the changes as
  trials and its verdict, in section 10, is to build it changed in six
  places; section 10 wins where it differs from the sections before. Its
  section 5 records each stage as it is built: by 13 November stages 1 to 4
  (the card, hands, money and the shelves; their pages are
  `docs/rules/scorecard.md`, `hands.md`, `money.md` and `shelves.md`).
- `architecture.md`: a review of the code's structure, made on 12 November 2026
  from five surveys of the three crates: what is in good shape and should be
  left alone, what slows or endangers the work, a verdict on four guesses made
  beforehand, and nine small changes in the order to make them (before the rules
  pass, beside the new island, after both), with those not worth making. Its
  answer is that no large refactor is due. It changes no code; its critic's
  corrections are marked where they stand, and the critic's order of work is
  its last section. By 13 November A1 (a golden test under the game's rules),
  A2's mends, A3's tests and A4 (with the towns' second pass) had landed, and
  B1 came with the new island's first stage.
