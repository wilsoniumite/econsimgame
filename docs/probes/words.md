# The game's words, counted

26 November 2026. A first count of the text a player can meet, made before a language and
interface pass (Stella's note of that day, at the foot of `docs/BACKLOG.md`). No code was
changed.

## How it was counted

Every string literal in `crates/isostate-viewer/src` and `crates/isostate-core/src`, tests
left out: 8,656. Files that write only for developers were dropped by name (the command line,
the lab, the sweeps, the shots, the names of people and places), and so were log lines and
option names. That leaves **3,295 strings and about 15,200 words in 72 files**.

The cut is by file, so some terminal text is still in. Read the figures as good to about a
tenth. The scripts are kept locally in `.claude/night/words/` (`strings.py REPO OUT.json`,
then `count.py OUT.json` and `senses.py`).

The game has no tooltips. Every sentence counted here is on a panel, a card, a toast or a
map caption.

## Size

- Sentences of four words or more: 1,341. Median 8 words. One in ten is over 18. 48 are over
  25 and 6 are over 40.
- Strings over 12 words: 330. Over 25 words: 87.

Where the long strings are:

| File | What it writes | Over 12 words |
|---|---|---|
| `edits.rs` (core) | what an edit did, and why one was refused | 52 of 232 |
| `overlay.rs` | map view captions, the line under the pointer | 34 of 197 |
| `sandbox.rs` | the tools' prompts | 28 of 116 |
| `game.rs` | the family game's cards | 27 of 164 |
| `chronicle.rs` (core) | the news | 27 of 216 |
| `inspect.rs` (core) | cards for a town, a works, a square, an idea | 15 of 299 |
| `family.rs` | the Family and Holdings panels | 12 of 126 |
| `rulebook.rs` | the Rules panel | 10 of 44 |
| `ui.rs` | the frame, Markets, News, the help card | 9 of 196 |

## The commonest words

town 153, year 150, hand 147, works 146, family 96, island 93, people 90, land 68, here 67,
square 64, share 57, market 56, made 56, none 51, ground 50, nothing 49, found 47, harvest 46.

Pairs: by hand 109, the family 65, the island 63, the island's 52, a year 40, the year's 32,
a town 30, a works 26. Threes: there is no 28, on the island 22, the year's turn 18.

## One word, several meanings

| Word | Count | Meanings |
|---|---|---|
| hand, hands | 147, 27 | an edit by the player ("by hand", 109); the player's tool ("the hand", 16, and the Hand tab); a worker ("hands", 27); cash ("in hand", 6); "materials to hand" |
| works | 146 | a firm of any kind, one or many ("a works", 26); also a verb |
| found | 83 | discovered, of an idea (59); established, of a town or a university (24). One button says "Found a town here", another "Discover it here" |
| trade | 44 | a skill a family knows; buying and selling ("the year's trade", "along the island's trade") |
| rule | 49 | a law of the simulation (the Rules tab); a drawn line ("a rule in the hand's ink", "ruled round in ink") |
| way | 34 | a route ("the ways", "the busiest way"); a method ("ways of making"); a distance ("the way to market") |
| raise | 25 | to mine ("coal raised"); to increase ("raised by a tenth") |
| stand | 69 | to exist ("works standing"); to be so ("as it stands"); to be allowed ("may stand"); "stands idle" |

## One thing, several words

| Thing | Words |
|---|---|
| The moment orders and edits take effect | the year's turn 18, the year's end 5, the year end 5, the turn 4, "in January", "To January" (a button) |
| The land that trades through a town | district 41, market area 7, "X's market" 11, "the country round X" (a card's title) |
| A map cell | square 91, land 71, ground 53, cell 21 |
| What a town knows | idea 32, method 18, technique 6, way of making or working 5, trade, "a use of the tasks rule" 4 |
| Coming to know it | found, discovered 10, known 28, pinned 12, "a draw" 8 |
| A firm | works 146, branch 18, pit 18, working 11, plant 10, firm 5, "shops and trades" |
| Moving goods | carting, carriage, carters, carried 51; road 29; the ways 15; traffic 14; haul 11 |
| Per | "a month", "a year", "a head" 65; "per cent" 7 beside "%" 21 |

## Turns of phrase

| Habit | Count | Example |
|---|---|---|
| A description where a name exists | throughout | "What a wage buys (1750 = 1)" for the real wage; "What is made a head" for output per head; "Living in towns" for the urban share; "how full it runs" for utilisation; "the island's money" for the money supply |
| A colon that joins two sentences | 321 | "No pit is opened by the hand: the island's adventurers find it at the next year's review of the land, and sink a working where one would pay, as they do on the island's own ground." |
| Possessives | 300 | "the island's" 52; "the town's own new works"; "the works' table"; "the woods' rule" |
| Saying what is not so | 119 | nothing, none, nobody, nowhere; "there is no" 28; "This record does not…" 11 |
| Pointing words | 184 | there 92, here 67, "of it" and "of them" 25: "works of it", "those who arrive" |
| Things given a posture | 114 | stand 69, kept 45: "The ledger, as it stands in 1851"; "What could stand on the square is still being asked for" |
| One rule explained by another | scattered | "as any idea does" 5; "where any closing works' money goes"; "as the island's own ground of the kind is" |
| The simulator's insides | scattered | "a draw", "a pin", "the tasks rule", "This record does not keep…", "the works' table is full", `people.enabled`, `edits::SWITCHABLE`. The Rules panel lists code names: `engel`, `longrun`, `transition`, `gauges` |
| Keys that are clauses | the inspector's cards | "its price", "the way to market", "rent bid for it now", "this year on its farmland", under the headings "Whose" and "Ways" |
| Words where a symbol would do | throughout | "up 7%", "down 40%", "4,000 to 4,400", "about 23% a year", "a tenth more" |

## What reads well already

- The names of goods and kinds of works (`data/content.toml`, `data/game.toml`): Bloomery,
  Colliery, Coke blast furnace, Town gas. Short, real, and each has an encyclopedia article.
- The Markets panel: a name, a bar and a number on each row.
- Short empty states: "Nothing yet. Press play."
- The news at its best: "Incomes there no longer keep a family: it buries more than it
  christens."
- "25 years' purchase": a period term that is also the correct term.

## What is wrong (Claude's reading)

1. **The text describes where it could name.** A label has to be a name: short, the same
   every time, something a reader can look up. The text avoids the names of things (real
   wage, utilisation, firm, discovery) and describes them again at each use. So one thing has
   several wordings, one word has several meanings, and nothing can be scanned.
2. **Each sentence tries to be complete.** It carries the rule, its exception and its reason
   together. That suits a page of rules and does not suit a panel. The colons, the "none"s
   and the long sentences come from here.
3. **It reports what the simulator did.** A player wants to know what happened on the
   island.
4. **One register serves every surface.** A label, a tooltip and a news item need three.

## Where the voice comes from

The helpers' briefs ask for it: "Plain, warm English … full sentences"
(`.claude/night/arch_review.js` and others). The commit messages show it settling on
11 November, the day the helpers began in number. Every helper since has read documents in
this voice and written more of it. A pass therefore needs written rules for interface text
that every brief carries, or the old voice grows back.

## Not decided here

The house rules for interface text, which word wins in each row above, and the order of the
pass. They are for Stella and Claude to settle on a sample panel (`docs/BACKLOG.md`, "The
language and interface pass").
