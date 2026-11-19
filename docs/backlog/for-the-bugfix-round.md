# Faults for the bugfix round
Faults found since 19 November 2026 that wait for a round of mending, one to a line.

- script.rs settle: the first picture taken right after settle can show an empty map, because the renderer has not drawn yet; seen in the first take of 0-tray-at-rest.png (from night/g-sandbox-1-hand, 2026-11-19)
- Season lags the date: two January pictures a few seconds apart show midwinter and green (1-harvest.png against 2-a-year-on.png) (from night/g-sandbox-1-hand, 2026-11-19)
- Run on is about 3 years a second, short of the design's 4 to 8 (script-runon.log) (from night/g-sandbox-1-hand, 2026-11-19)
- In a sandbox the inspector still offers the family's Buy/Build and the family card stays top left; the design leaves these for stages 4 and 5 (4-people-halved.png) (from night/g-sandbox-1-hand, 2026-11-19)
- The town of Brackness on island 7 has its church, a small works and its town ground drawn on pale shallow water inside the railway embankment (seen in the standard picture works-1900-close; with the tree it is the busiest works district, 18 works) (from night/g-tree-land, 2026-11-19)
- On island 2 with the tree, card.rs gives freight times weight-and-path-cost at 1.107 of the carting bill less handling in 1750, outside its 0.1 tolerance; the test the_cards_facts_are_what_they_say plays without the tree and the cause was not traced (from night/g-tree-land, 2026-11-19)
- Steel rail mill never built by 1900 on 4 of 8 islands with the tree; engineering works and ammonia-soda on 2 each (from night/g-tree-land, 2026-11-19)
- Emigrants rise from 17.4 to 63.3 a run with the tree, with one island at 280 (from night/g-tree-land, 2026-11-19)
- Canal km in 1800 falls from 712 to 523 with the tree (from night/g-tree-land, 2026-11-19)
- The books' copy of the island grows to 4.3 MB with the tree; the bound in tests/accounts.rs was raised from 3 MB to 5 MB (from night/g-tree-land, 2026-11-19)
- The square card's line 'the way to market 0.424, as many squares of open lowland' reads stiffly (crates/isostate-core/src/inspect.rs, Market group); seen in the printed card for the square at 43,75 (from night/g-sandbox-2-tell, 2026-11-19)
