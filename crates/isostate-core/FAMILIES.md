# Families and the live API

The company half of the family game, in the core: families that own works and
land and grow through their people; the player's family, which chooses only
from short lists; rival AI families that play by the same rules; and a live
API so the viewer can run the simulation in a background thread and draw each
year as it is simulated. None of it is in the Python.

Everything is off by default. With `n_families = 0` the state has no families
and no owner tables, every hook returns at once, and a run is byte for byte the
same as the port of the Python (checked: `series_monthly.csv`, `events.json`,
`params.json` and `run.bin` are identical apart from the date in the header,
and all parity tests pass unchanged). Turn families on with `n_families` in the
parameters, or `isostate-core run --families 6`.

## The design in brief

- **Members and posts.** Each member lives in a town and has a post: Child
  (under 14), Learning (an apprenticeship or university, until a year),
  Manager of a trade, Head, or Unposted. There is one head, at the seat.
- **Training at 14.** A decision: an apprenticeship in a trade at a town (four
  years; the family learns the trade and the member manages it a quarter
  better), or university (four years, `university_gain` of skill, educated).
  The options are the first trade at the seat, each other trade in the town
  where it is strongest and the family has nobody in it yet, and university.
  The default is the first trade at the seat while nobody living is trained or
  training in it; after that, what an AI family would choose: the most
  valuable trade (estates aside) the family neither knows nor is learning,
  where it is strongest, and once it has every trade, a trade in the town where
  it is worth most and the family has nobody yet. So a family left to its
  defaults spreads its children over trades and towns; they do not all end up
  managing the seat's first trade.
- **Trades.** The six industry groups (iron and steel, textiles, coal and
  mining, engineering, building materials, estates). A family starts with one
  and learns more only when apprentices finish. Shops and trades belong to no
  group.
- **Branches.** A trade in a town where the family has someone to run it: a
  manager of that trade living there, or the head at the seat for every known
  trade. Each branch Grows (founds one works a year, two under a runner of
  skill above 0.6, by the prudent rules below; estates branches buy farmland
  instead), Holds, or Winds down (sells one works a year, the smallest first,
  to the town's savers at book value; estates: one farm to the gentry).
- **Bonus.** A family's works under a branch need `1 - bonus` of the inputs,
  heat and labor per unit: `family_bonus` (0.08) times the manager's skill, or
  at the seat `family_seat_bonus` (0.12) times the head's. Land units on family
  cells get the bonus as extra productivity. A head under 18 gives none.
- **Absentee works.** A family works with no branch over it (its manager died
  or moved) runs under a hired manager: no bonus, and `absentee_cut` (20 per
  cent) of its profit goes to the town's households as his pay.
- **The seat.** Only at the seat does the player choose works and buy land
  directly: `found_works` refuses elsewhere and refuses unknown trades, and
  `buy_land` refuses cells outside the seat's reach (`may_buy_land` gives the
  reason, cash aside, for greying out a button); growing branches build and
  buy elsewhere. Moving the seat takes effect at the next year end; the head
  moves with it.
- **Posting.** Any adult not training and not the head can be posted to
  manage a known trade in any town, or left unposted. A move to another town
  takes effect at the next year end.
- **Children.** Each adult of 22 to 42 has a child with a yearly chance of
  `family_birth_rate` (0.15) while the family has fewer than ten living
  members. A child's skill is half its parent's and half drawn.
- **Succession.** When the head dies, the eldest adult heads the family at
  once; the player gets a decision to choose another adult instead. With no
  adults the eldest member heads without a bonus until 18.
- **Decisions** wait in `pending`; an unanswered one takes its default at the
  end of the year after it was raised (the viewer pauses on new ones). The
  player's first decision, its trade, is due at the end of the second year,
  because a viewer simulates the first year before the player sees the map;
  until it is answered every branch of the player's holds, so nothing is built
  or bought in a trade the player then drops (teenagers apprenticed in the
  provisional trade switch with it).
- **Money.** Dividends of family works go to the family, rents of family land
  too; the family spends `family_cons_income` (0.7) of last month's income
  plus `family_cons_wealth` (1 per cent) of its cash a month, never more than
  its cash, through its seat town's landlords. Land and works are not drawn
  on, so a landed family saves a quarter to a third of its rents. It backs its
  own young and growing works. Every flow is a transfer, `econ::money_total`
  includes family cash, and the monthly conservation check holds.
- **The start**, after the warm-up: every family gets `family_start_cash` of
  the money stock from the towns' savers. The player and the landed rivals get
  the farmland nearest their seat until they hold `family_start_share` (3 per
  cent) of all rent; industrial rivals get the largest works of their seat's
  strongest trade up to the same share of all works' capital. The player's
  first trade is its first decision, defaulting to the trade with the best
  prospects at the seat (`families::prospects`, estates counting the land
  yield).
- **AI families** answer their own decisions (a gifted child to university if
  nobody is educated, otherwise the player's default after the first
  apprentice: a trade they neither know nor are learning, where it is
  strongest), post every idle adult to run a known trade where it is worth
  most, keep every branch growing, and never move their seat.
- **Prudent rules** for every growing branch: only methods whose output would
  be at most `family_max_share` (0.25) of the town's demand and whose price is
  at most `family_max_price_ratio` (3) times its base, relative to the town's
  typical good; the best expected returns above the hurdle rate first; and a
  cash reserve of `family_reserve_months` of spending.

## The API

All in `isostate_core::families`, the main ones re-exported at the crate root.
Actions return `Result` with a readable error.

```rust
pub fn pending(st: &State, family: usize) -> Vec<Decision>
pub fn decide(st: &mut State, C: &Content, family: usize, id: u64, option: usize) -> Result<(), String>
pub fn post_member(st: &mut State, family: usize, member: usize, posting: Posting) -> Result<(), String>
pub fn set_stance(st: &mut State, family: usize, group: Group, town: usize, stance: Stance) -> Result<(), String>
pub fn move_seat(st: &mut State, family: usize, town: usize) -> Result<(), String>
pub fn site_options(st: &State, C: &Content, P: &Params, family: usize, cell: usize) -> Result<Vec<SiteOption>, String>
pub fn found_works(st: &mut State, C: &Content, P: &Params, family: usize, method: usize, cell: usize, year: i64, events: &mut Vec<Event>) -> Result<usize, String>
pub fn buy_land(st: &mut State, C: &Content, P: &Params, family: usize, cell: usize) -> Result<f64, String>
pub fn may_buy_land(st: &State, family: usize, cell: usize) -> Result<(), String>   // cash aside
pub fn family_report(st: &State, C: &Content, P: &Params, family: usize) -> Result<FamilyReport, String>
pub fn prospects(st: &State, C: &Content, P: &Params, town: usize) -> [f64; 6]   // best prudent expected return per trade
pub fn market_depth(st: &State, C: &Content, town: usize, method: usize, K: f64) -> (f64, f64, f64, f64, f64)
pub fn land_price(st: &State, P: &Params, cell: usize) -> f64
pub fn ownership_shares(st: &State, C: &Content) -> (f64, f64, f64)
```

Call them between years (after `sim::run_year`), with `year` = `sim.year` and
`events` = `&mut sim.events`. The types, all `Clone`, `Debug` and `Serialize`:

```rust
pub enum Group { Iron, Textiles, CoalMining, Engineering, Building, Estates }   // ALL, index(), name(), label()
pub enum Strategy { Player, Landed, Industrial }
pub enum Path { Apprentice { group: Group, town: usize }, University }
pub enum Post { Child, Learning { path: Path, until: i64 }, Manager { group: Group }, Head, Unposted }
pub enum Stance { Grow, Hold, WindDown }
pub enum Posting { Manager { group: Group, town: usize }, Unposted }
pub enum Choice { Trade { group: Group }, Apprentice { group: Group, town: usize }, University, Head { member: usize } }
pub struct DecisionOption { pub label: String, pub choice: Choice }
pub struct Decision { pub id: u64, pub prompt: String, pub member: Option<usize>, pub options: Vec<DecisionOption>, pub default: usize, pub due: i64 }
pub struct FamilyReport {
    pub index: usize, pub surname: String, pub player: bool, pub strategy: Strategy,
    pub seat: usize, pub seat_name: String, pub next_seat: Option<usize>, pub head: Option<usize>,
    pub known: Vec<Group>, pub first_trade: Group,
    pub cash: f64, pub works_value: f64, pub land_value: f64, pub net_worth: f64,
    pub last_year: Income, pub firms: Vec<usize>, pub absentee: Vec<usize>, pub cells: Vec<usize>,
    pub members: Vec<MemberReport>, pub branches: Vec<BranchReport>, pub pending: Vec<Decision>,
}
pub struct MemberReport {
    pub index: usize, pub first: String, pub age: i64, pub skill: f64, pub alive: bool,
    pub town: usize, pub town_name: String, pub post: Post, pub training_until: Option<i64>,
    pub educated: bool, pub trade: Option<Group>, pub next: Option<(usize, Post)>,
    pub born: i64, pub parent: Option<usize>,   // parent None for the founding generation
}
pub struct BranchReport {
    pub group: Group, pub town: usize, pub town_name: String, pub runner: Option<usize>,
    pub stance: Stance, pub bonus: f64, pub works: Vec<usize>, pub cells: usize, pub last_year_profit: f64,
}
pub struct Income { pub dividends: [f64; 6], pub other_dividends: f64, pub rents: f64, pub sales: f64, pub spending: f64, pub invested: f64 }
pub struct SiteOption { pub method: usize, pub name: String, pub label: String, pub group: Option<Group>, pub sums: Result<SiteSums, String>, pub allowed: Result<(), String> }
```

`SiteSums` is as before (gate price, inputs, heat, labor, margin, rent bid,
capital cost, expected return, bonus, agglomeration, and the market depth:
output, town demand, town output, market share, price ratio). The live API
(`live_static`, `LiveCursor::new`, `live_year`) is unchanged; `LiveYear::families`
carries the new `FamilyReport`s.

**News** (kind "family", `Event::family` set): births, a child turning
fourteen, the end of a training, a new trade, deaths, successions, the
player's works opening and its sales when winding down, all at weight 0.6 for
the player. Rivals are heard of only for big moves: a new trade, a head's death
and the succession (0.3), and their first works of each kind (0.35).

## Is there a game? (v2)

`examples/bots.rs` plays the player's family through the public API with six
families and the defaults, on seeds 1, 2, 3 and 7:

- **passive**: takes every default.
- **landlord**: starts in estates and buys the land around its seat with the
  best rent per price, keeping a quarter of its cash.
- **founder**: the defaults, plus the best works it may found at the seat each
  year in a deep enough market.
- **thoughtful**: starts in the trade with the best prospects at the seat;
  trains children in the trades with the best prospects where they are
  strongest (a gifted child to university first); posts every idle adult to
  run a known trade where its prospects are best; grows the branches whose
  prospects clear the hurdle rate and holds the rest; founds works at the seat
  like the founder; and buys land around the seat like the landlord.

Net worth in 1775, 1800, 1850 and 1900, the richest rival's in 1900, the
player's rank among the six families (1 is richest), the share of its income
the family saved in its first ten years, and the works it founded (still
standing in 1900), its trades and its branches at the end:

| Seed | Strategy | 1775 | 1800 | 1850 | 1900 | Rival 1900 | Rank | Saved | Works, trades, branches |
|---|---|---|---|---|---|---|---|---|---|
| 1 | passive | 1.6m | 2.5m | 8.2m | 28.1m | 29.6m | 2/3/2/2 | 27% | 1099 (810), 6, 13 |
| 1 | landlord | 1.8m | 4.3m | 20.4m | 44.7m | 32.2m | 1/1/1/1 | 32% | 975 (699), 6, 12 |
| 1 | founder | 1.6m | 3.0m | 7.7m | 23.9m | 33.3m | 2/2/2/2 | 27% | 1171 (872), 6, 14 |
| 1 | thoughtful | 1.7m | 4.1m | 12.6m | 40.2m | 32.3m | 1/1/1/1 | 32% | 1211 (888), 5, 11 |
| 2 | passive | 1.3m | 2.7m | 7.8m | 22.1m | 20.2m | 3/1/1/1 | 27% | 867 (663), 6, 11 |
| 2 | landlord | 1.5m | 3.3m | 11.9m | 30.7m | 18.1m | 2/1/1/1 | 32% | 865 (708), 6, 10 |
| 2 | founder | 1.3m | 2.8m | 8.8m | 21.8m | 16.2m | 3/1/1/1 | 27% | 1089 (818), 6, 10 |
| 2 | thoughtful | 1.5m | 3.1m | 13.7m | 25.9m | 17.4m | 2/1/1/1 | 32% | 906 (625), 4, 9 |
| 3 | passive | 1.2m | 2.7m | 6.8m | 16.1m | 16.5m | 2/1/4/3 | 27% | 894 (728), 6, 8 |
| 3 | landlord | 1.3m | 3.2m | 10.5m | 26.2m | 18.9m | 2/1/1/1 | 32% | 813 (663), 6, 10 |
| 3 | founder | 1.2m | 2.8m | 7.3m | 18.3m | 17.6m | 2/1/2/1 | 27% | 1049 (831), 6, 10 |
| 3 | thoughtful | 1.3m | 3.2m | 10.1m | 27.6m | 18.4m | 2/1/1/1 | 32% | 948 (752), 6, 10 |
| 7 | passive | 1.4m | 2.4m | 6.3m | 17.2m | 19.2m | 4/2/4/4 | 27% | 988 (769), 6, 9 |
| 7 | landlord | 1.7m | 3.2m | 11.9m | 23.3m | 19.6m | 3/1/1/1 | 34% | 904 (698), 6, 11 |
| 7 | founder | 1.4m | 2.4m | 9.6m | 19.3m | 20.1m | 4/2/1/2 | 27% | 1026 (819), 6, 8 |
| 7 | thoughtful | 1.6m | 2.8m | 10.1m | 20.9m | 23.0m | 4/2/1/3 | 34% | 1072 (726), 5, 9 |

- Passive play now plays as a landed rival does: its children branch out into
  the trades and towns it lacks and every branch grows by the prudent rules.
  It ends second, first, third and fourth in 1900, within a tenth of the
  richest rival each time, and never last.
- A family left to its defaults saves 27 per cent of its income in its first
  ten years, and its cash between years would pay for the best prudent works
  at its seat, in any trade, from the first year in every seed (such works
  cost a hundred to a few thousand; a farm costs about ten thousand).
- The thoughtful strategy wins three seeds of four, 17 to 71 per cent ahead
  of the passive player in 1900, and comes third in seed 7.
- The landlord wins all four and beats the thoughtful player in three: buying
  the land around the seat with three quarters of the cash each year is the
  strongest single lever. Founding works at the seat is about even with
  letting the seat's estates branch buy farms: the founder ends within 15 per
  cent of the passive player either way.

What got it here: the player starts as landed gentry like the landed rivals
(a 33k cash start left even a perfect player hopelessly behind estates worth
thirty times as much); families spend 70 per cent of their income and 1 per
cent of their cash a month but nothing of their land or works (spending out
of net worth, tried before, had a landed family spending all its rents and
never able to build at the seat); growing branches keep three months of
spending back; direct land purchases are confined to the seat's reach; and the
default training spreads the children over the trades and towns the family
lacks, as an AI family's does, instead of making them all managers of the
seat's first trade.

`prospects` for one town takes about 0.6 ms in a release build and 1 ms in a
debug build (in 1753, nine towns), so ranking every town once a year costs 10
to 20 ms in the live thread.

## What worries me

- **Land beats works.** The landlord comes first in every seed. Land returns
  4 per cent plus the growth of rents, safely; family works are small (40 per
  cent of full size), many die young, and a branch founds at most two a year.
  If the game is to be about people and trades, land wants to be dearer (more
  years' purchase), or direct purchases capped at a few cells a year like an
  estates branch's, or priced up as one buyer takes a town's land.
- **Families own most of the industry by 1900.** Saving a quarter to a third
  of their rents with every branch growing, the six families hold 22 per cent
  of the works' capital in 1800 and 57 per cent in 1900 in the default run
  (from a few per cent before). If that is too much: fewer works per branch a
  year, or a lower share saved.
- **Prospects mislead as percentages.** The 150 to 210 per cent for iron in
  every town in the 1770s is crucible steel: a works of about 200 capital with
  a margin of 22 on a price of 61, at last year's prices. Not a bug: the
  model's own entrants see the same snapshot and keep founding them (184
  standing in 1773 in seed 7). But such works are tiny next to a family worth
  millions: a player who goes by the percentages is drawn to them, and the
  thoughtful bot, which trains, posts and holds by them, does worse than the
  landlord. A hint in money a year (the return times the capital a trade can
  take in a town) would steer a player better.
- **The default first trade is usually estates**, because the land yield beats
  the prudent industrial prospects at the seat in 1750. Children trained by
  default bring the other trades in from the second apprentice on.
- **Industrial rivals are feeble.** They start with works worth 3 per cent of
  all works' capital, which is little next to the landed families' land, and
  end under a million in 1900 against 17 to 25 million for the landed.
- **Decisions come often.** Every child turning fourteen is a pause for the
  player, about one every few years with births at 0.15 a year per adult.
- **AI families never wind down, move their seat or hold**, and post idle
  adults to wherever a trade is worth most, not where it is growing.
- **Hostile buyouts remain**: `buy_land` still takes a rival's land at its
  price without consent, now only around the seat.

## How much families own

The section above measures the families against each other. Across the whole
economy (`examples/live.rs`, seed 7) the six families hold 7, 22, 48 and 57 per
cent of the works' capital in 1750, 1800, 1850 and 1900; 14 to 26 per cent of
the rent; and 2 to 10 per cent of the money, their cash piling up once their
branches cannot spend it all. Land, at 25 years' purchase, is two thirds of a
landed family's wealth: 12 to 19 million of its 18 to 25 million in 1900.

## Surprises and earlier findings

- **A new town's land is priced as if labor were free for a year.** In the
  Python (and so in the port), a town founded at the end of a year has a mean
  wage of zero for that year, which `land.update_land_use` clamps to 1e-6; the
  land bids around it come out near 1e19 for a year. `land_price` therefore
  uses the rent each cell actually paid over the last year
  (`State::cell_rent_paid`, `State::cell_rent_year`). Worth fixing in the
  Python (give a new town its donor's wage in `wbar`).
- **Expected returns are snapshots.** At last year's prices and 85 per cent
  utilisation; they hold for deep markets (textile mills realise most of what
  they promise) and fail for small labor-heavy works and thin markets
  (machinery and rails asked for long before anyone makes them, priced at many
  times base). Works die young for everyone, not just families: in a run
  without families the median life of the closed works founded after 1750 is
  11 years. Hence the market-depth fields and the prudent rules.
- **Thin-market harvesting** (merchants pay the gate price for all output,
  bought or not) is closed for the player by allowing works only at the seat,
  in known trades, and for the AI by the prudent rules; growing branches follow
  the prudent rules too.
- **Values count works under construction**, and family spending shows up in
  the landlords' spending series because it goes through their basket.
