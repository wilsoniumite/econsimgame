# Words on the screen

Rules for the text a player reads in the game. Stella asked for them on 26 November 2026
and agreed to their outline the same day (her note is at the foot of `docs/BACKLOG.md`).
The count that led here is `docs/probes/words.md`.

These rules are for the interface only. Documents and comments keep their own voice.

## What she asked for

- A matter-of-fact, descriptive tone.
- Terms, numbers and symbols on the interface. Sentences go into tooltips.
- Tooltips inside tooltips, which lock after a short time.
- Links of several kinds in different colours. One kind opens Wikipedia.
- The tooltips form a graph a player can learn from: the game, the simulation, economics.
- No tutorial where curiosity can do the job.

## Three registers

| Register | Where | Form |
|---|---|---|
| Label | panels, cards, buttons, charts, the map | a term, a number, a symbol |
| Tooltip | behind every label and number | one to three short sentences, then rows |
| News | the News panel and its toasts | a headline and one or two sentences |

## Labels

1. One to three words. A noun or a number. A button is a verb: Build, Close.
2. Use the standard term: the word an economist or the trade uses, the word a reader can
   look up. Real wage, utilisation, market area.
3. One term for one thing, everywhere. The glossary, `data/words.toml`, is the list. A new
   label takes a term from it or adds one.
4. Modern terms on the frame. The frame serves 1750 and 2050 alike. Period words belong in
   names (Bloomery, Colliery, Handloom weaving) and in the news.
5. Sentence case. No article, no possessive, no full stop.
6. Numbers in digits, thousands separated, three significant figures unless money needs
   pence. Units as symbols: `£`, `%`, `/mo`, `/yr`, `×`, `→`, `▲`, `▼`.
7. A label that needs explaining gets a tooltip. It does not get longer.

## Tooltips

The parts, always in this order: the term; its value now, if it has one; one to three
sentences; rows, if the number is made of parts; links.

1. A sentence has at most 18 words. A tooltip has at most three sentences.
2. Present tense, active voice, subject first.
3. The first sentence says what the thing is. The next says what moves it, or what it
   decides.
4. One fact to a sentence. No colon or semicolon joining two sentences. No brackets.
5. Name the thing again where a pronoun could be misread. Avoid "there", "here" and "of it".
6. Say what is so. Use a negative only when the absence is the news: "No workers".
7. Describe the island, never the program. No draw, pin, record, switch or table.
8. A number made of parts is shown as rows, never as a sentence.
9. A tooltip opens from data the view already holds. It never waits for the simulation.

## Links

Three kinds, told apart by colour and by line.

| Kind | Looks | Does |
|---|---|---|
| A term of the game | blue, dotted line | hovering opens its tooltip |
| A thing on the island: a town, a firm, a family, a good | brown, solid line | clicking goes to it |
| Wikipedia | plum, with `↗` | clicking opens the browser |

- The first mention of a glossary term in a tooltip is a link. The game finds it; nobody
  marks it by hand.
- Our own sentence comes first. Wikipedia is for the reader who wants more.
- We write our own definitions. We do not copy Wikipedia's text.
- A Wikipedia link is an article title kept in the glossary. No link is built from text
  made while the game runs.

## The news

1. A headline of up to eight words. One or two sentences under it.
2. Period words are welcome here, and so is a turn of phrase.
3. Nouns still come from the glossary, so that they link.
4. What an item needed in order to be told goes in the tooltip of its label, not in its
   text.

## Words we use

| Thing | Term | Not |
|---|---|---|
| A business at one site | firm | works, concern |
| A person employed; the input | worker; labour | hand |
| A change the player makes | edit, marked `✎` | by hand, the hand |
| Learning a technology | discovered | found |
| Starting a town, a firm, a university | founded | |
| The land that trades in one town's market | market area | district, the country round |
| The ground a firm, a mine or a block of houses occupies | site | square, cell |
| 1 January, when orders and edits take effect | 1 January | the year's turn, the year's end |
| What a town knows; one way to make one good | technology; method | idea, technique, way of making |
| Moving goods: its cost, a line on the map, how much | transport, route, traffic | carting, carriage, the ways |
| What is made | output | made, raised |
| What a month of work pays; the same against prices | wage; real wage | pay, what a wage buys |
| How much a firm can make; its machines | capacity; equipment | plant |
| Costing most, costing least | highest price, lowest price | dearest, cheapest |
| For each | `/mo`, `/yr`, per head | a month, a year, a head |

This table is a first list. Stella may strike any row. A changed row is changed in
`data/words.toml`, in one place.

## Words kept out of labels and tooltips

stands, stood, standing (of things); as it stands; kept; told; asked; whatever; whoever;
the island's; by hand; the year's turn; a head; per cent; dear; of it; there is no.

## How the rules are held

A test reads `data/words.toml` and the cards built from it. It checks the length of labels
and sentences, the words kept out, that every link leads somewhere, and that every term
has a definition. A piece that adds interface text runs it.
