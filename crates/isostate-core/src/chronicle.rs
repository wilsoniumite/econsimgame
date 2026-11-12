//! The chronicle: news that reads like a history. Not in the Python; see
//! `docs/rules/chronicle.md`.
//!
//! The observer and the families between them file well over a thousand items
//! in a game, and the viewer used to call everything of weight 0.6 or more
//! "major". With `chronicle.enabled` this module does five things, none of
//! which the simulation ever reads:
//!
//!   - the works a family's branches open on their own are told once, when a
//!     trade first reaches a town, and otherwise in one summary a year; the
//!     family's everyday news weighs less than its great moments, and a
//!     succession is one item;
//!   - every observer event says why it is news (`Event::why`), and so do
//!     hunger and large closures;
//!   - the few items of each decade that a history would keep are marked
//!     (`Event::kept`), year by year in a live game and again in `sim::finish`;
//!   - towns and districts get stories of their own: a coalfield boom, a town
//!     in decline, one town overtaking another, the fastest-growing town of a
//!     decade, a district whose works close together, a decade of new towns;
//!   - three small faults of the observer are mended (later towns get their
//!     milestones, doublings are measured from the first few years and not
//!     from 1750 alone, a rival's "first of its kind" is its first ever).
//!
//! Switched off, every function here returns at once and `State::chron` stays
//! empty, so a port run's `events.json` does not change by a byte.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::families::{self, Group};
use crate::mat::Mat;
use crate::observer::MILESTONE_CITY;
use crate::params::Params;
use crate::state::{Event, State};
use crate::util::{fmt_pct, fmt_thousands, fmt_thousands_f0, pairwise_sum};

// ---------------------------------------------------------------- numbers

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ChronicleParams {
    /// The news is kept as a history: provenance, ranking, regional stories.
    pub enabled: bool,

    // ---- ranking
    /// At most this many items of a decade are kept.
    pub per_decade: usize,
    /// A finished decade keeps at least this many, if it has them above `fill_floor`.
    pub at_least: usize,
    /// At most this many of one kind in a decade (slumps called and dated count
    /// as one kind, as do railways planned and opened).
    pub per_kind: usize,
    /// Inventions are what the period is remembered for: they may have more.
    pub inventions: usize,
    /// An item lighter than this is kept only to fill a thin decade.
    pub floor: f64,
    /// And one lighter than this is never kept.
    pub fill_floor: f64,

    // ---- the player's own news
    /// Weight of the family's great moments: a new head, a new trade, the head's death.
    pub own_great: f64,
    /// Weight of its everyday news: births, birthdays, trainings, sales.
    pub own_small: f64,
    /// Weight of the yearly summary of what its branches opened.
    pub own_summary: f64,

    // ---- regional stories
    /// Doublings are measured from the mean of this many first years.
    pub base_years: i64,
    /// A coalfield booms when it raises `boom_growth` times the coal of
    /// `boom_years` earlier and at least `boom_share` of the island's. One
    /// and a half times is the line beside the game's other rules: with the
    /// merchants' prices a district's coal no longer swings with the iron
    /// trade, and the 2 this was first set at, under the port, is passed less
    /// than once a run (docs/rules/chronicle.md, "In company").
    pub boom_years: i64,
    pub boom_growth: f64,
    pub boom_share: f64,
    /// A town is in decline when it is `decline_fall` below a peak at least
    /// `decline_years` old.
    pub decline_years: i64,
    pub decline_fall: f64,
    /// A town has overtaken another when it has been `overtake_margin` ahead of
    /// it for `overtake_years` running.
    pub overtake_years: i64,
    pub overtake_margin: f64,
    /// The decade's fastest-growing town is news if it outgrew the island by this much.
    pub fastest_margin: f64,
    /// A district's works close together when at least `closures_min` shut in
    /// a year and they are `closures_share` of those that stood (in a usual
    /// year a district loses one or two in a hundred). Five, where it was ten
    /// before the life cycle for works: a district then has some forty works
    /// where it had four hundred, and ten of them hardly ever shut in a year
    /// (docs/rules/works.md, "In company with the rest of the game").
    pub closures_min: usize,
    pub closures_share: f64,
    /// A decade of new towns is told when at least this many were founded in it.
    pub new_towns_min: usize,
    /// Towns smaller than this (the town and its district) have no stories.
    pub min_people: f64,
    /// Years before the same town can have the same story again.
    pub quiet_years: i64,
}

impl Default for ChronicleParams {
    /// Off: what the port runs with.
    fn default() -> ChronicleParams {
        ChronicleParams {
            enabled: false,
            per_decade: 4,
            at_least: 2,
            per_kind: 1,
            inventions: 2,
            floor: 0.6,
            fill_floor: 0.45,
            own_great: 0.7,
            own_small: 0.5,
            own_summary: 0.45,
            base_years: 5,
            boom_years: 5,
            boom_growth: 1.5,
            boom_share: 0.15,
            decline_years: 8,
            decline_fall: 0.1,
            overtake_years: 3,
            overtake_margin: 0.05,
            fastest_margin: 0.1,
            closures_min: 5,
            closures_share: 0.1,
            new_towns_min: 3,
            min_people: 5000.0,
            quiet_years: 15,
        }
    }
}

impl ChronicleParams {
    /// The game's chronicle.
    pub fn game() -> ChronicleParams {
        ChronicleParams { enabled: true, ..ChronicleParams::default() }
    }
}

// ---------------------------------------------------------------- provenance

/// Why an item is news: which test fired, on what, with which numbers.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Why {
    /// The detector's test, in a few words.
    pub test: String,
    /// The series or the town's quantity it read.
    pub series: String,
    /// What it read this year.
    pub value: f64,
    /// What that had to pass.
    pub threshold: f64,
    /// What it was compared with (an earlier level, a trend, a peak), if anything.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<f64>,
    /// The same in a sentence.
    pub note: String,
}

/// The provenance of an event, or nothing while the chronicle is off. The
/// numbers are rounded to six figures: they are for reading.
pub fn why(P: &Params, test: &str, series: &str, value: f64, threshold: f64, baseline: Option<f64>, note: String) -> Option<Why> {
    if !P.chronicle.enabled {
        return None;
    }
    Some(Why { test: test.to_string(), series: series.to_string(), value: sig6(value), threshold: sig6(threshold), baseline: baseline.map(sig6), note })
}

fn sig6(x: f64) -> f64 {
    if !x.is_finite() || x == 0.0 {
        return x;
    }
    let scale = 10f64.powi(5 - x.abs().log10().floor() as i32);
    // a number too small to scale is left as it is
    if scale.is_finite() && scale > 0.0 { (x * scale).round() / scale } else { x }
}

// ---------------------------------------------------------------- memory

/// What the chronicle remembers between years (`State::chron`).
#[derive(Clone, Debug, Default)]
pub struct ChronicleState {
    /// (family, method) of every works a family has ever held.
    pub kinds: BTreeSet<(usize, usize)>,
    /// (family, trade, town) wherever a family has ever had works.
    pub reached: BTreeSet<(usize, usize, usize)>,
    /// The works the player's branches opened this year, and whether each was told on its own.
    pub branch_opened: Vec<(usize, bool)>,
    /// The works the player's branches extended this year, and what each extension cost
    /// (empty without the life cycle for works).
    pub branch_extended: Vec<(usize, f64)>,
    /// For each of `MILESTONE_CITY`, the towns that have passed it and the year.
    pub city_passed: Vec<Vec<(usize, i64)>>,
    /// One entry a year since the start, each a value per town: people in the
    /// town and its district, people in the town itself, coal raised, the
    /// year's wage, and net arrivals from other towns.
    pub pop: Vec<Vec<f64>>,
    pub urban: Vec<Vec<f64>>,
    pub coal: Vec<Vec<f64>>,
    pub wage: Vec<Vec<f64>>,
    pub moved: Vec<Vec<f64>>,
    /// The year's mean prices by town and good, one matrix a year.
    pub prices: Vec<Mat>,
    /// Each town's greatest population so far and its year.
    pub peak: Vec<(i64, f64)>,
    /// Whether the fall from that peak has been told.
    pub decline_told: Vec<bool>,
    /// The island's first and second towns as last announced.
    pub leaders: [Option<usize>; 2],
    /// Who has led each place since, and for how many years running.
    pub challenger: [Option<usize>; 2],
    pub lead_years: [i64; 2],
    /// (story, town) -> the year it was last told.
    pub told: BTreeMap<(String, usize), i64>,
    /// For each town, the last year a founding redrew its district (its own, or
    /// that of a new town cut out of it), or the first year: nothing about a
    /// town is compared across that.
    pub redrawn: Vec<i64>,
}

/// The families' holdings at the start count as held, so a works given at the
/// start is not "the first of its kind" when a second is built.
pub fn init(st: &mut State, C: &Content, P: &Params) {
    if !P.chronicle.enabled {
        return;
    }
    note_holdings(st, C);
}

fn note_holdings(st: &mut State, C: &Content) {
    if !families::active(st) {
        return;
    }
    for f in 0..st.F_count {
        let Some(k) = families::owner_of_firm(st, f) else { continue };
        note_works(st, C, k, f);
    }
}

fn note_works(st: &mut State, C: &Content, family: usize, f: usize) {
    let m = st.f_method[f];
    st.chron.kinds.insert((family, m));
    if let Some(g) = Group::of_method(C, m) {
        st.chron.reached.insert((family, g.index(), st.f_town[f]));
    }
}

// ---------------------------------------------------------------- the families' works

/// Whether `method` would be the family's first works of its kind: the port
/// looks at what it owns today, so a rival that sold its only one has a
/// "first" again; the chronicle remembers.
pub fn first_of_kind(st: &State, P: &Params, family: usize, method: usize) -> bool {
    !P.chronicle.enabled || !st.chron.kinds.contains(&(family, method))
}

/// The weight of one of the player's own items: its great moments are those a
/// rival's would be news for too (`great`).
pub fn own_weight(P: &Params, great: bool) -> f64 {
    if !P.chronicle.enabled {
        0.6
    } else if great {
        P.chronicle.own_great
    } else {
        P.chronicle.own_small
    }
}

/// A family works has just been founded (`families::found_at`). Returns true
/// if the chronicle has dealt with the news, so the port's item is not made:
/// that is the case for a works one of the player's branches opened on its
/// own. It is told at once if it takes the trade to a town where the family
/// never had it, and otherwise waits for the year's summary.
#[allow(clippy::too_many_arguments)]
pub fn works_founded(st: &mut State, C: &Content, P: &Params, family: usize, fid: usize, by_hand: bool, cost: f64, roi: f64, year: i64, events: &mut Vec<Event>) -> bool {
    if !P.chronicle.enabled {
        return false;
    }
    let (m, i) = (st.f_method[fid], st.f_town[fid]);
    let group = Group::of_method(C, m);
    let new_here = group.is_some_and(|g| !st.chron.reached.contains(&(family, g.index(), i)));
    note_works(st, C, family, fid);
    if family != 0 || by_hand {
        return false;
    }
    if new_here {
        let surname = &st.families[family].surname;
        let trade = group.map_or("a new trade".to_string(), |g| g.label().to_lowercase());
        let mut e = Event::new(
            year as f64 + 0.95,
            "family",
            0.6,
            format!("The {surname} family sets up in {trade} at {}", st.town_name[i]),
            format!(
                "{} is the first works the family has had in that trade there. It is built for {} and expects to return {:.0} per cent a year.",
                st.f_name[fid],
                fmt_thousands_f0(cost),
                100.0 * roi
            ),
        );
        e.town = Some(i);
        e.cell = Some(st.f_cell[fid]);
        e.firm = Some(fid);
        e.family = Some(family);
        events.push(e);
    }
    st.chron.branch_opened.push((fid, new_here));
    true
}

/// A branch of a family has ordered more capacity for a works it has
/// (`works::extend_for_family`: with the life cycle for works a growing branch
/// extends before it founds). Returns true if the chronicle has dealt with the
/// news, so the works' own item is not made: an extension is the branch's own
/// doing, as a works it opens is, and waits for the year's summary. Without
/// this a player's branches file some 240 such items in a game, which is the
/// flood over again.
pub fn works_extended(st: &mut State, P: &Params, family: usize, fid: usize, cost: f64) -> bool {
    if !P.chronicle.enabled || family != 0 {
        return false;
    }
    st.chron.branch_extended.push((fid, cost));
    true
}

/// One item for all that the player's branches did on their own this year
/// (`families::yearly`, after the branches have run): the works they opened,
/// unless each was told on its own, and the works they extended. It is light:
/// it belongs in the family's news, not the island's.
pub fn branch_summary(st: &mut State, C: &Content, P: &Params, family: usize, year: i64, events: &mut Vec<Event>) {
    if !P.chronicle.enabled || family != 0 {
        return;
    }
    let opened = std::mem::take(&mut st.chron.branch_opened);
    let extended = std::mem::take(&mut st.chron.branch_extended);
    // works that were each told on their own are not told again
    let opened: Vec<usize> = if opened.iter().all(|&(_, told)| told) { Vec::new() } else { opened.iter().map(|&(f, _)| f).collect() };
    if opened.is_empty() && extended.is_empty() {
        return;
    }
    let surname = &st.families[family].surname;
    let trade = |f: usize| Group::of_method(C, st.f_method[f]).map_or("other trades".to_string(), |g| g.label().to_lowercase());
    // "at Melbourne two in iron and steel and one in textiles; at Granness one in
    // engineering": towns in the order the branches ran, trades in the order they built
    let by_town = |works: &[usize]| -> String {
        let mut towns: Vec<(usize, Vec<(String, usize)>)> = Vec::new();
        for &f in works {
            let t = st.f_town[f];
            let at = match towns.iter().position(|(town, _)| *town == t) {
                Some(at) => at,
                None => {
                    towns.push((t, Vec::new()));
                    towns.len() - 1
                }
            };
            let name = trade(f);
            match towns[at].1.iter_mut().find(|(n, _)| *n == name) {
                Some((_, n)) => *n += 1,
                None => towns[at].1.push((name, 1)),
            }
        }
        let per_town: Vec<String> = towns
            .iter()
            .map(|(t, trades)| {
                let list: Vec<String> = trades.iter().map(|(name, n)| format!("{} in {name}", number(*n))).collect();
                format!("at {} {}", st.town_name[*t], and_list(&list))
            })
            .collect();
        per_town.join("; ")
    };
    let grown: Vec<usize> = extended.iter().map(|&(f, _)| f).collect();
    let laid_out: f64 = extended.iter().map(|&(_, cost)| cost).sum();
    let (title, text) = match (opened.len(), grown.len()) {
        (1, 0) => {
            let f = opened[0];
            (
                format!("The {surname} branch at {} opens a works", st.town_name[st.f_town[f]]),
                format!("{}, in {}, expects to return {:.0} per cent a year.", st.f_name[f], trade(f), 100.0 * st.f_reason[f].roi),
            )
        }
        (n, 0) => (format!("The {surname} branches open {} works", number(n)), format!("{}.", capital(&by_town(&opened)))),
        (0, 1) => {
            let f = grown[0];
            (
                format!("The {surname} branch at {} extends a works", st.town_name[st.f_town[f]]),
                format!("{}, in {}: more capacity is ordered for {}.", st.f_name[f], trade(f), fmt_thousands_f0(laid_out)),
            )
        }
        (0, n) => (
            format!("The {surname} branches extend {} works", number(n)),
            format!("{}. More capacity is ordered for {} in all.", capital(&by_town(&grown)), fmt_thousands_f0(laid_out)),
        ),
        (a, b) => (
            format!("The {surname} branches open {} works and extend {}", number(a), number(b)),
            format!("{}. Extended, for {} in all: {}.", capital(&by_town(&opened)), fmt_thousands_f0(laid_out), by_town(&grown)),
        ),
    };
    let first = *opened.first().or(grown.first()).expect("something was opened or extended");
    let mut e = Event::new(year as f64 + 0.95, "family", P.chronicle.own_summary, title, text);
    e.town = Some(st.f_town[first]);
    e.family = Some(family);
    events.push(e);
}

/// The head of a family who died this year, as `families::people` knew him
/// when it wrote his burial: which of the year's items is the burial, and the
/// facts the succession tells of him. Handed over as facts, so that the
/// succession need not read them back out of the burial's sentence, and the
/// burial's wording can change without the succession going quiet.
pub struct Burial {
    /// The burial's place among the year's items.
    pub item: usize,
    /// His first name and surname.
    pub who: String,
    pub age: i64,
    /// The town he is buried at.
    pub place: String,
    /// Whether that town is the family's seat.
    pub at_seat: bool,
}

/// A head of the player's family has died and another has taken over
/// (`families::people`, with the year's family news before it is filed). The
/// port files two items of the same weight, and a decade's place for the
/// family went to the burial every time. Here the succession is the item a
/// history keeps, and says whom the new head (`first`, by first name) follows;
/// the burial stays in the family's own news. The last item is the
/// succession; `burial` says which item is the burial and what to tell of
/// him. Without one (a cousin who takes the name on when nobody is left buries
/// nobody) nothing is done.
pub fn succession(P: &Params, family: usize, items: &mut [Option<Event>], first: &str, burial: Option<&Burial>) {
    if !P.chronicle.enabled || family != 0 {
        return;
    }
    let Some(Burial { item, who, age, place, at_seat }) = burial else { return };
    let Some((Some(heads), before)) = items.split_last_mut() else { return };
    let Some(Some(buried)) = before.get_mut(*item) else { return };
    heads.text = if *at_seat {
        format!("{first} follows {who}, who died at {age} and is buried at {place}, the family's seat.")
    } else {
        format!("{first} follows {who}, who died at {age} and is buried at {place}. {}", heads.text)
    };
    buried.weight = P.chronicle.own_small;
}

// ---------------------------------------------------------------- the observer's faults

/// The level a doubling is measured from. The port takes the first year alone;
/// here it is the mean of the first `base_years`, and nothing doubles while
/// that mean is still forming. `when` and `v0` are the port's.
pub fn baseline(P: &Params, series: &[f64], y0: i64, year: i64, when: i64, v0: f64) -> Option<(String, f64)> {
    if !P.chronicle.enabled {
        return Some((when.to_string(), v0));
    }
    let n = P.chronicle.base_years.max(1);
    if year < when + n {
        return None;
    }
    let v: Vec<f64> = (when..when + n).map(|y| crate::observer::annual(series, y0, y)).filter(|x| x.is_finite() && *x > 0.0).collect();
    if v.is_empty() {
        return None;
    }
    let label = if n == 1 { when.to_string() } else { format!("{when}\u{2013}{:02}", (when + n - 1) % 100) };
    Some((label, pairwise_sum(&v) / v.len() as f64))
}

/// Cities passing round numbers, for every town and not only the first
/// (`observer::yearly` calls this in place of its own loop). The first town
/// past a size is told as the port tells it; a later one more quietly, with
/// the towns that got there before it.
pub fn city_milestones(st: &mut State, P: &Params, year: i64, events: &mut Vec<Event>, start_year: i64) {
    let mut passed = std::mem::take(&mut st.chron.city_passed);
    passed.resize(MILESTONE_CITY.len(), Vec::new());
    for i in 0..st.T_max {
        if !st.town_active[i] {
            continue;
        }
        let u = st.urban_pop[i];
        for (idx, &thr) in MILESTONE_CITY.iter().enumerate() {
            if u < thr as f64 || passed[idx].iter().any(|&(t, _)| t == i) {
                continue;
            }
            let before = passed[idx].clone();
            passed[idx].push((i, year));
            if year == start_year {
                continue;
            }
            let name = &st.town_name[i];
            let size = fmt_thousands(thr);
            let (weight, text) = if before.is_empty() {
                (0.55 + 0.05 * idx as f64, format!("{name} is the first town in the country with more than {size} inhabitants."))
            } else {
                let earlier: Vec<String> = before
                    .iter()
                    .map(|&(t, y)| if y == start_year { st.town_name[t].clone() } else { format!("{} ({y})", st.town_name[t]) })
                    .collect();
                let after = if earlier.len() <= 3 { format!(", after {}", and_list(&earlier)) } else { String::new() };
                (0.45 + 0.05 * idx as f64, format!("{name} is the {} town on the island with more than {size} inhabitants{after}.", ordinal(before.len() + 1)))
            };
            let mut ev = Event::new(year as f64 + 0.75, "milestone", weight, format!("{name} passes {size} people"), text);
            ev.town = Some(i);
            ev.cell = Some(st.town_node[i]);
            ev.why = why(
                P,
                "a town passes a round number",
                &format!("people living in {name} itself"),
                u,
                thr as f64,
                None,
                format!("{} people live in the town itself; {} had passed {size} before it", fmt_thousands_f0(u), number(before.len())),
            );
            events.push(ev);
        }
    }
    st.chron.city_passed = passed;
}

// ---------------------------------------------------------------- the year

/// The chronicle's year end, after the observer's (`sim::run_year`): the
/// towns' numbers are noted, the regional stories told, and the decades ranked
/// again with what the year added.
pub fn yearly(st: &mut State, C: &Content, P: &Params, year: i64, events: &mut Vec<Event>) {
    if !P.chronicle.enabled {
        return;
    }
    let mut ch = std::mem::take(&mut st.chron);
    note_year(st, C, &mut ch, year);
    if let Some(coal) = C.good_name.iter().position(|n| n == "coal") {
        coalfields(st, C, P, &mut ch, coal, year, events);
    }
    declines(st, P, &mut ch, year, events);
    overtakings(st, C, P, &mut ch, year, events);
    closures(st, C, P, &mut ch, year, events);
    // the decades are the calendar's, whatever year the run starts in: a run
    // from 1755 has a short first decade, 1755 to 1759, and then the 1760s
    if decade_ends(year) {
        fastest_of_decade(st, C, P, &ch, year, events);
        new_towns_of_decade(st, P, year, C.start_year, events);
    }
    st.chron = ch;
    note_holdings(st, C);
    closure_provenance(st, C, P, year, events);
    rank(events, P, C.start_year, C.end_year, year);
}

/// The calendar decade a year falls in, by its first year: 1750 for 1750 to 1759.
fn decade_of(year: i64) -> i64 {
    year.div_euclid(10) * 10
}

/// Whether a calendar decade ends with this year (1759, 1769, ...).
fn decade_ends(year: i64) -> bool {
    year.rem_euclid(10) == 9
}

fn note_year(st: &State, C: &Content, ch: &mut ChronicleState, year: i64) {
    let T = st.T_max;
    let on = |v: &[f64]| -> Vec<f64> { (0..T).map(|t| if st.town_active[t] { v[t] } else { 0.0 }).collect() };
    ch.pop.push(on(&st.pop));
    ch.urban.push(on(&st.urban_pop));
    ch.wage.push(on(&st.wbar));
    let coal = C.good_name.iter().position(|n| n == "coal");
    ch.coal.push((0..T).map(|t| coal.map_or(0.0, |g| st.town_prod_year[(t, g)])).collect());
    ch.prices.push(st.pbar.clone());
    // people's moves between towns this year (`people::migration` keeps them by
    // the towns that stood before this year's were founded)
    let mut moved = vec![0.0; T];
    let act = movers(st, year);
    if let Some(act) = &act {
        let flow = &st.migration_flow;
        for (a, &i) in act.iter().enumerate() {
            for (b, &j) in act.iter().enumerate() {
                moved[i] -= flow[(a, b)];
                moved[j] += flow[(a, b)];
            }
        }
    }
    ch.moved.push(moved);
    ch.peak.resize(T, (year, 0.0));
    ch.decline_told.resize(T, false);
    ch.redrawn.resize(T, year);
    // a new town takes people and pits from the districts it is cut out of, so
    // nothing about it or about them is compared across its founding: their
    // peaks start again, and so do the island's places if one of them held or
    // was about to take one. The towns it did not touch keep their stories.
    let cut = redrawn_towns(st, year);
    for t in (0..T).filter(|&t| cut[t]) {
        ch.redrawn[t] = year;
        ch.peak[t] = (year, 0.0);
    }
    if cut.iter().any(|&c| c) {
        let mut order: Vec<usize> = (0..T).filter(|&t| st.town_active[t]).collect();
        order.sort_by(|&a, &b| st.pop[b].total_cmp(&st.pop[a]).then(a.cmp(&b)));
        let placed = ch.leaders.iter().chain(&ch.challenger).flatten().any(|&t| cut[t]);
        if placed || order.iter().take(2).any(|&t| cut[t]) {
            ch.leaders = [None; 2];
        }
    }
    for t in 0..T {
        if st.town_active[t] && st.pop[t] > ch.peak[t].1 {
            ch.peak[t] = (year, st.pop[t]);
            ch.decline_told[t] = false;
        }
    }
}

/// The towns whose districts a founding redrew this year: each new town, and
/// every town that held some of its ground the year before. Those are the
/// towns `people::resettle_new_towns` takes its people from.
///
/// Under the game's rules for people (docs/rules/people.md) households go with
/// their cells every year, founding or no founding: a road or a railway can
/// carry a district from one market to another, and a town's people jump when
/// it does. A town that gained or lost `REDRAWN_SHARE` of its people that way
/// in the year is redrawn too.
fn redrawn_towns(st: &State, year: i64) -> Vec<bool> {
    let T = st.T_max;
    let new: Vec<bool> = (0..T).map(|t| st.town_active[t] && st.town_founded[t] == year).collect();
    let mut cut = new.clone();
    if new.contains(&true) {
        for (&now, &was) in st.catch.iter().zip(&st.prev_catch) {
            if now >= 0 && was >= 0 && (now as usize) < T && (was as usize) < T && new[now as usize] {
                cut[was as usize] = true;
            }
        }
    }
    for t in 0..T.min(st.pp_moved.len()) {
        cut[t] = cut[t] || (st.town_active[t] && st.pp_moved[t] >= REDRAWN_SHARE);
    }
    cut
}

/// A town is redrawn when this share of its people came or went with their
/// cells in a year. It is the least fall the chronicle calls a decline
/// (`decline_fall`) and the least lead it calls outgrowing the island
/// (`fastest_margin`): a smaller redrawing cannot make a story by itself.
pub const REDRAWN_SHARE: f64 = 0.1;

/// The towns `st.migration_flow` is indexed by, if it is this year's.
fn movers(st: &State, year: i64) -> Option<Vec<usize>> {
    let act: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t] && st.town_founded[t] < year).collect();
    (st.migration_flow.rows == act.len() && st.migration_flow.cols == act.len() && act.len() >= 2).then_some(act)
}

/// What a town is known for: the good of which it supplies the largest share
/// of the island's, if that is a fair share. Shops and trades aside.
fn known_for(st: &State, C: &Content, t: usize) -> String {
    let share = |g: usize| -> f64 {
        let total: f64 = (0..st.T_max).map(|i| st.town_prod_year[(i, g)]).sum();
        if total > 0.0 { st.town_prod_year[(t, g)] / total } else { 0.0 }
    };
    let best = (0..C.G).filter(|&g| C.good_name[g] != "services").max_by(|&a, &b| share(a).total_cmp(&share(b)).then(b.cmp(&a)));
    match best {
        Some(g) if share(g) >= 0.15 => format!(" {} supplies {} of the island's {}.", st.town_name[t], fmt_pct(share(g), 0), C.good_label[g].to_lowercase()),
        _ => String::new(),
    }
}

/// A town younger than this has no stories of booms or closures: its first
/// years are a settling-in.
const YOUNG: i64 = 5;

/// Whether `story` was told of town `t` too lately to be told again.
fn quiet(ch: &ChronicleState, P: &Params, story: &str, t: usize, year: i64) -> bool {
    ch.told.get(&(story.to_string(), t)).is_some_and(|&last| year - last < P.chronicle.quiet_years)
}

/// " Kelgate and Ashdelf are booming too.": the towns a story could have led with and did not.
fn and_others(st: &State, towns: &[usize], what: &str) -> String {
    if towns.is_empty() {
        return String::new();
    }
    let names: Vec<String> = towns.iter().map(|&t| st.town_name[t].clone()).collect();
    format!(" {} {what}.", and_list(&names))
}

/// "grown by 12%", "shrunk by 4%".
fn changed(x: f64) -> String {
    if x >= 0.005 {
        format!("grown by {}", fmt_pct(x, 0))
    } else if x <= -0.005 {
        format!("shrunk by {}", fmt_pct(-x, 0))
    } else {
        "stayed as it was".to_string()
    }
}

/// A coalfield boom: the pits around a town raise several times the coal of a
/// few years earlier, and a fair share of the island's. One story a year, led
/// by the largest field; a district that a founding redrew inside those years
/// is passed over, because its pits are not the pits it had.
fn coalfields(st: &State, C: &Content, P: &Params, ch: &mut ChronicleState, coal: usize, year: i64, events: &mut Vec<Event>) {
    let Q = &P.chronicle;
    let k = ch.coal.len() - 1;
    let back = Q.boom_years.max(1) as usize;
    let was = year - back as i64;
    if k < back {
        return;
    }
    let total = pairwise_sum(&ch.coal[k]);
    if total <= 0.0 {
        return;
    }
    let mut fields: Vec<usize> = (0..st.T_max)
        .filter(|&t| {
            let (now, then) = (ch.coal[k][t], ch.coal[k - back][t]);
            st.town_active[t] && st.town_founded[t] <= was - YOUNG && ch.redrawn[t] <= was && now > 0.0 && now >= Q.boom_growth * then && now >= Q.boom_share * total && !quiet(ch, P, "coal", t, year)
        })
        .collect();
    fields.sort_by(|&a, &b| ch.coal[k][b].total_cmp(&ch.coal[k][a]).then(a.cmp(&b)));
    let Some(&t) = fields.first() else { return };
    for &f in &fields {
        ch.told.insert(("coal".to_string(), f), year);
    }
    let (now, then) = (ch.coal[k][t], ch.coal[k - back][t]);
    let name = &st.town_name[t];
    let grew = if then <= 0.0 {
        format!("coal where none was raised in {was}")
    } else if now / then >= 20.0 {
        // a pit or two five years ago: the multiple would be a number nobody can picture
        format!("many times the coal they raised in {was}")
    } else {
        format!("{:.1} times the coal they raised in {was}", now / then)
    };
    // what carries it away, if that is new
    let carried = if P.railways.enabled {
        // the game's railways (docs/rules/railways.md): a line with no second town
        // may be a branch to farms, woods or works as well as to pits, or the town's
        // own first line, so the promoters' book is asked what the line was for
        carried_by(name, was, crate::railways::line_for_coal(st, t, was as f64), "A branch line to the collieries", &st.town_name)
    } else {
        carried_by(name, was, port_line_for_coal(st, t, was as f64), "A mineral line", &st.town_name)
    };
    // and what the price says about why
    let (p1, p0) = (ch.prices[k][(t, coal)], ch.prices[k - back][(t, coal)]);
    let dp = if p0 > 0.0 { p1 / p0 - 1.0 } else { 0.0 };
    let price = if dp > 0.05 {
        format!(" Coal fetches {} more there than in {was}: buyers want more than the pits can raise.", fmt_pct(dp, 0))
    } else if dp < -0.05 {
        format!(" Coal is {} cheaper there than in {was}: the new pits have outrun their buyers.", fmt_pct(-dp, 0))
    } else {
        format!(" Its price has hardly moved since {was}: the new pits found buyers as fast as they opened.")
    };
    let mine = C.sector_name.iter().position(|n| n == "coalmine");
    let mut ev = Event::new(
        year as f64 + 0.9,
        "region",
        0.72,
        format!("Coal boom around {name}"),
        format!(
            "The pits around {name} raise {grew}, and {} of all the island's.{carried}{price}{}",
            fmt_pct(now / total, 0),
            and_others(st, &fields[1..], if fields.len() == 2 { "is booming too" } else { "are booming too" })
        ),
    );
    ev.town = Some(t);
    // the pit with most hands. Not `ex_out`: the land was shared out again at the
    // year's end and the units listed afresh, so last month's output no longer
    // lines up with them; the labour is carried over to the new list.
    let hands = |u: usize| st.ex_Lprev.get(u).copied().unwrap_or(0.0);
    ev.cell = mine
        .and_then(|s| (0..st.ex_cell.len()).filter(|&u| st.ex_town[u] == t && st.ex_sector[u] == s).max_by(|&a, &b| hands(a).total_cmp(&hands(b)).then(b.cmp(&a))))
        .map(|u| st.ex_cell[u])
        .or(Some(st.town_node[t]));
    ev.why = why(
        P,
        "coal raised against a few years before",
        &format!("coal raised around {name} in the year"),
        now,
        Q.boom_growth * then,
        Some(then),
        format!("{now:.0} against {then:.0} in {was} (it needs {} times as much), and {} of the island's (it needs {})", Q.boom_growth, fmt_pct(now / total, 0), fmt_pct(Q.boom_share, 0)),
    );
    events.push(ev);
}

/// `railways::line_for_coal` for the port's railways (`State::rail_opened`):
/// a mineral spur has no second town, and every other line runs between two.
fn port_line_for_coal(st: &State, t: usize, since: f64) -> Option<crate::railways::CoalLine> {
    let reaches = |r: &crate::state::RailOpened| r.b >= 0 && (r.a == t || r.b == t as i64);
    let lines = &st.rail_opened;
    let k = lines.iter().rposition(|r| r.year >= since && (r.a == t || r.b == t as i64))?;
    let r = &lines[k];
    if r.b < 0 {
        return Some(crate::railways::CoalLine { opened: r.year, colliery: true, from: None, nth: 0, first: r.year });
    }
    let to_town: Vec<f64> = lines[..=k].iter().filter(|m| reaches(m)).map(|m| m.year).collect();
    let other = if r.a == t { r.b as usize } else { r.a };
    Some(crate::railways::CoalLine { opened: r.year, colliery: false, from: (other != t).then_some(other), nth: to_town.len(), first: to_town[0] })
}

/// What a coal boom says carries the coal away: a branch to the pits, or a
/// line to the town. "The railway that reached" a town is said only of its
/// first line; a later one is named by where it comes from, with the first
/// beside it if that too came since `was` (docs/probes/news-read.md, fault 2).
fn carried_by(name: &str, was: i64, line: Option<crate::railways::CoalLine>, branch: &str, towns: &[String]) -> String {
    let Some(l) = line else { return String::new() };
    let (opened, first) = (l.opened.floor(), l.first.floor());
    let from = l.from.and_then(|b| towns.get(b));
    if l.colliery {
        format!(" {branch}, opened in {opened}, brings it down from the pits.")
    } else if l.nth <= 1 {
        format!(" The railway that reached {name} in {opened} carries it away.")
    } else if l.first >= was as f64 {
        let from = from.map_or(String::new(), |b| format!(", from {b},"));
        format!(" The railway reached {name} in {first}, and a {} line{from} in {opened}; they carry it away.", ordinal(l.nth))
    } else {
        match from {
            Some(b) => format!(" The line from {b}, opened in {opened}, carries it away."),
            None => format!(" A {} line to {name}, opened in {opened}, carries it away.", ordinal(l.nth)),
        }
    }
}

/// A town in decline: well below a peak it reached years ago. One story a
/// year, led by the town that has lost most people. A town's peak is counted
/// from the last year a founding redrew its district, and a new town that
/// never grew past the people it was founded with is not said to decline: it
/// did not take.
fn declines(st: &State, P: &Params, ch: &mut ChronicleState, year: i64, events: &mut Vec<Event>) {
    let Q = &P.chronicle;
    let mut towns: Vec<usize> = (0..st.T_max)
        .filter(|&t| {
            let (py, peak) = ch.peak[t];
            st.town_active[t] && !ch.decline_told[t] && py > st.town_founded[t] && peak >= Q.min_people && year - py >= Q.decline_years && st.pop[t] <= (1.0 - Q.decline_fall) * peak
        })
        .collect();
    towns.sort_by(|&a, &b| (ch.peak[b].1 - st.pop[b]).total_cmp(&(ch.peak[a].1 - st.pop[a])).then(a.cmp(&b)));
    let Some(&t) = towns.first() else { return };
    for &d in &towns {
        ch.decline_told[d] = true;
    }
    let (py, peak) = ch.peak[t];
    let now = st.pop[t];
    let name = &st.town_name[t];
    // where its people go: the town most of this year's leavers chose, and what
    // drew them, which is what `people::migration` weighs
    let mut reason = String::new();
    if let Some(act) = movers(st, year) {
        if let Some(a) = act.iter().position(|&i| i == t) {
            let flow = &st.migration_flow;
            let best = (0..act.len()).filter(|&b| b != a).max_by(|&x, &y| flow[(a, x)].total_cmp(&flow[(a, y)]).then(y.cmp(&x)));
            if let Some(b) = best.filter(|&b| flow[(a, b)] > 0.0) {
                let j = act[b];
                let ratio = st.y_ratio[j] / st.y_ratio[t].max(1e-3);
                let further = if ratio >= 10.0 {
                    "many times as far".to_string()
                } else if ratio >= 3.0 {
                    format!("more than {} times as far", number(ratio.floor() as usize))
                } else if ratio >= 2.0 {
                    "more than twice as far".to_string()
                } else if ratio >= 1.01 {
                    format!("{} further", fmt_pct(ratio - 1.0, 0))
                } else {
                    "a little further".to_string()
                };
                reason = format!(" Its people leave for {}, where a household's income goes {further}.", st.town_name[j]);
            }
        }
    }
    if reason.is_empty() {
        let g = P.growth_slope * st.y_ratio[t].max(1e-3).ln() + P.growth_base;
        reason = if g < 0.0 {
            " Incomes there no longer keep a family: it buries more than it christens.".to_string()
        } else {
            " It has been losing its people to the towns around it.".to_string()
        };
    }
    let mut ev = Event::new(
        year as f64 + 0.9,
        "region",
        0.68,
        format!("{name} in decline"),
        format!(
            "{name} and its district had {} people {}in {py}. There are {} now, {} fewer.{reason}{}",
            fmt_thousands_f0(peak),
            // the count a district starts with, or is left with by a founding, is no peak it climbed to
            if py == ch.redrawn[t] { "" } else { "at their peak " },
            fmt_thousands_f0(now),
            fmt_pct(1.0 - now / peak, 0),
            and_others(st, &towns[1..], if towns.len() == 2 { "has lost people too" } else { "have lost people too" })
        ),
    );
    ev.town = Some(t);
    ev.cell = Some(st.town_node[t]);
    ev.why = why(
        P,
        "people against the town's own peak",
        &format!("people in {name} and its district"),
        now,
        (1.0 - Q.decline_fall) * peak,
        Some(peak),
        format!("{} below the peak of {py}, which is {} years old (it needs {} below a peak at least {} years old)", fmt_pct(1.0 - now / peak, 0), year - py, fmt_pct(Q.decline_fall, 0), Q.decline_years),
    );
    events.push(ev);
}

/// One town overtaking another to become the island's first or second, by
/// the people of the town and its district: only once it has been clear of the
/// other by `overtake_margin` for `overtake_years` running.
fn overtakings(st: &State, C: &Content, P: &Params, ch: &mut ChronicleState, year: i64, events: &mut Vec<Event>) {
    let Q = &P.chronicle;
    let mut order: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
    order.sort_by(|&a, &b| st.pop[b].total_cmp(&st.pop[a]).then(a.cmp(&b)));
    if order.len() < 3 {
        return;
    }
    let (Some(first), Some(second)) = (ch.leaders[0], ch.leaders[1]) else {
        // the first year, or a founding has just redrawn a leading town's district: nothing to compare with
        ch.leaders = [Some(order[0]), Some(order[1])];
        ch.challenger = [None; 2];
        ch.lead_years = [0; 2];
        return;
    };
    let k = ch.pop.len() - 1;
    for place in 0..2 {
        let holder = if place == 0 { first } else { ch.leaders[1].unwrap_or(second) };
        // the largest town not already at this place or above it
        let rival = order.iter().copied().find(|&t| t != holder && (place == 0 || Some(t) != ch.leaders[0]));
        let ahead = rival.filter(|&t| st.pop[t] >= (1.0 + Q.overtake_margin) * st.pop[holder]);
        if ahead.is_none() || ahead != ch.challenger[place] {
            ch.challenger[place] = ahead;
            ch.lead_years[place] = 0;
        }
        let Some(now) = ahead else { continue };
        ch.lead_years[place] += 1;
        if ch.lead_years[place] < Q.overtake_years {
            continue;
        }
        ch.challenger[place] = None;
        ch.lead_years[place] = 0;
        if place == 0 {
            // the old first town is now the second: the same story, told once
            ch.leaders = [Some(now), Some(holder)];
            ch.challenger[1] = None;
            ch.lead_years[1] = 0;
        } else {
            ch.leaders[1] = Some(now);
        }
        let (a, b) = (&st.town_name[now], &st.town_name[holder]);
        // how each has done in ten years, or since a founding last redrew either's district
        let back = ((year - ch.redrawn[now].max(ch.redrawn[holder])).max(0) as usize).min(10).min(k);
        let then = &ch.pop[k - back];
        let grown = |t: usize| -> String { if then[t] > 0.0 { changed(st.pop[t] / then[t] - 1.0) } else { "grown from nothing".to_string() } };
        // "first", not "largest": the towns are ranked with their districts, and
        // the town itself may be the smaller of the two (the city milestones
        // count the people inside the town)
        let (title, weight) = if place == 0 {
            (format!("{a} overtakes {b} as the island's first town"), 0.8)
        } else {
            (format!("{a} overtakes {b} as the island's second town"), 0.7)
        };
        let since = if back > 0 { format!(" In {} years {a} has {} and {b} has {}.", number(back), grown(now), grown(holder)) } else { String::new() };
        let mut ev = Event::new(
            year as f64 + 0.9,
            "region",
            weight,
            title,
            format!(
                "{a} and its district have {} people to {b}'s {}.{since}{}",
                fmt_thousands_f0(st.pop[now]),
                fmt_thousands_f0(st.pop[holder]),
                known_for(st, C, now)
            ),
        );
        ev.town = Some(now);
        ev.cell = Some(st.town_node[now]);
        ev.cell2 = Some(st.town_node[holder]);
        ev.why = why(
            P,
            "one town's people against another's",
            &format!("people in {a} and its district"),
            st.pop[now],
            (1.0 + Q.overtake_margin) * st.pop[holder],
            Some(st.pop[holder]),
            format!("{a} has been at least {} ahead of {b} for {} years running (it needs {})", fmt_pct(Q.overtake_margin, 0), Q.overtake_years, Q.overtake_years),
        );
        events.push(ev);
        if place == 0 {
            break;
        }
    }
}

/// A district whose works close together: many in one year, and a good share
/// of those that stood. Shops and trades are not counted. One story a year,
/// led by the district that lost the largest share.
fn closures(st: &State, C: &Content, P: &Params, ch: &mut ChronicleState, year: i64, events: &mut Vec<Event>) {
    let Q = &P.chronicle;
    let works = |f: usize| C.method_name[st.f_method[f]] != "services";
    let mut closed: Vec<Vec<usize>> = vec![Vec::new(); st.T_max];
    for &f in st.closed_year.iter().filter(|&&f| works(f)) {
        closed[st.f_town[f]].push(f);
    }
    let mut stood: Vec<usize> = closed.iter().map(|c| c.len()).collect();
    for f in (0..st.F_count).filter(|&f| st.f_alive[f] && works(f)) {
        stood[st.f_town[f]] += 1;
    }
    let share = |t: usize| closed[t].len() as f64 / stood[t].max(1) as f64;
    // a town in its first years is still finding what it can make (and whatever
    // `closures_min` is set to, a district where nothing shut has no story)
    let mut towns: Vec<usize> = (0..st.T_max)
        .filter(|&t| st.town_active[t] && st.town_founded[t] <= year - YOUNG && closed[t].len() >= Q.closures_min.max(1) && share(t) >= Q.closures_share && !quiet(ch, P, "closures", t, year))
        .collect();
    towns.sort_by(|&a, &b| share(b).total_cmp(&share(a)).then(a.cmp(&b)));
    let Some(&t) = towns.first() else { return };
    for &d in &towns {
        ch.told.insert(("closures".to_string(), d), year);
    }
    let n = closed[t].len();
    let name = &st.town_name[t];
    // what most of them made, and what became of its price
    let mut by_good: BTreeMap<usize, usize> = BTreeMap::new();
    for &f in &closed[t] {
        *by_good.entry(C.method_out[st.f_method[f]]).or_insert(0) += 1;
    }
    let (&g, &ng) = by_good.iter().max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0))).expect("some closed");
    let good = C.good_label[g].to_lowercase();
    let makers = if ng == n { format!("all of them makers of {good}") } else { format!("{} of them makers of {good}", number(ng)) };
    let k = ch.prices.len() - 1;
    let mut reason = " They could no longer cover their costs.".to_string();
    if k >= 3 {
        let fall = |years: usize| 1.0 - ch.prices[k][(t, g)] / ch.prices[k - years][(t, g)].max(1e-9);
        let dw = ch.wage[k][t] / ch.wage[k - 1][t].max(1e-9) - 1.0;
        // what the commonest kind among them spends most on, and what that costs now
        let mut by_method: BTreeMap<usize, usize> = BTreeMap::new();
        for &f in closed[t].iter().filter(|&&f| C.method_out[st.f_method[f]] == g) {
            *by_method.entry(st.f_method[f]).or_insert(0) += 1;
        }
        let m = *by_method.iter().max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0))).expect("some made it").0;
        let spend = |x: usize| C.m_in[(m, x)] * ch.prices[k][(t, x)];
        let input = (0..C.G).filter(|&x| x != g && spend(x) > 0.0).max_by(|&a, &b| spend(a).total_cmp(&spend(b)).then(b.cmp(&a)));
        let dearer = input.map_or(0.0, |x| ch.prices[k][(t, x)] / ch.prices[k - 1][(t, x)].max(1e-9) - 1.0);
        if fall(1) > 0.05 {
            reason = format!(" The price of {good} there is {} lower than a year ago.", fmt_pct(fall(1), 0));
        } else if fall(3) > 0.05 {
            reason = format!(" The price of {good} there is {} lower than three years ago.", fmt_pct(fall(3), 0));
        } else if dearer > 0.05 {
            let x = input.expect("an input grew dearer");
            reason = format!(" The {} they work up costs {} more there than a year ago, and the price of {good} has not followed.", C.good_label[x].to_lowercase(), fmt_pct(dearer, 0));
        } else if dw > 0.05 {
            reason = format!(" Wages there are up {} in a year, and the price of {good} has not followed.", fmt_pct(dw, 0));
        }
    }
    let biggest = closed[t].iter().copied().max_by(|&a, &b| st.f_K[a].total_cmp(&st.f_K[b]).then(b.cmp(&a))).expect("some closed");
    let named = format!(" The largest was {}, open since {}.", st.f_name[biggest], st.f_born[biggest]);
    let mut ev = Event::new(
        year as f64 + 0.9,
        "region",
        0.66,
        format!("{} works close around {name}", capital(&number(n))),
        format!(
            "{} of the {} works in and around {name} shut in {year}, {makers}.{reason}{named}{}",
            capital(&number(n)),
            stood[t],
            and_others(st, &towns[1..], "had a hard year too")
        ),
    );
    ev.town = Some(t);
    ev.cell = Some(st.f_cell[biggest]);
    ev.why = why(
        P,
        "works closed in one district in a year",
        &format!("works closed around {name}, shops and trades aside"),
        n as f64,
        (Q.closures_min as f64).max(Q.closures_share * stood[t] as f64),
        Some(stood[t] as f64),
        format!("{n} of {} closed (it needs at least {} and {} of those that stood)", stood[t], Q.closures_min, fmt_pct(Q.closures_share, 0)),
    );
    events.push(ev);
}

/// The town that grew fastest over the decade just ended, if it outgrew the
/// island by a margin. A town whose district a founding redrew in those years
/// is not in the running.
fn fastest_of_decade(st: &State, C: &Content, P: &Params, ch: &ChronicleState, year: i64, events: &mut Vec<Event>) {
    let Q = &P.chronicle;
    let k = ch.pop.len() - 1;
    // from the end of the decade before, or from the first year in the first decade
    let k0 = k.saturating_sub(10);
    if k == k0 {
        return;
    }
    let from = year - (k - k0) as i64;
    let (all1, all0) = (pairwise_sum(&ch.pop[k]), pairwise_sum(&ch.pop[k0]));
    if all0 <= 0.0 {
        return;
    }
    let island = all1 / all0 - 1.0;
    let best = (0..st.T_max)
        .filter(|&t| st.town_active[t] && ch.redrawn[t] <= from && ch.pop[k0][t] > 0.0 && ch.pop[k0][t] >= Q.min_people)
        .max_by(|&a, &b| (ch.pop[k][a] / ch.pop[k0][a]).total_cmp(&(ch.pop[k][b] / ch.pop[k0][b])).then(b.cmp(&a)));
    let Some(t) = best else { return };
    let (p1, p0) = (ch.pop[k][t], ch.pop[k0][t]);
    let g = p1 / p0 - 1.0;
    // (a town that shrank less than the rest is not the fastest-growing)
    if g < island + Q.fastest_margin || g <= 0.0 {
        return;
    }
    let name = &st.town_name[t];
    let arrived: f64 = (k0 + 1..=k).map(|y| ch.moved[y][t]).sum();
    let share = arrived / (p1 - p0);
    let source = if share >= 0.5 {
        " More than half of its new people moved there from other towns."
    } else if share >= 0.3 {
        " About a third of its new people moved there from other towns."
    } else if share >= 0.15 {
        " About a fifth of its new people moved there from other towns."
    } else if arrived < 0.0 {
        " It grew by its own families, while more people left than came."
    } else {
        " Nearly all of it is the town's own families growing."
    };
    let decade = decade_of(year);
    let mut ev = Event::new(
        year as f64 + 0.9,
        "region",
        0.62,
        format!("{name} is the fastest-growing town of the {decade}s"),
        format!(
            "{name} and its district grew from {} people to {}, by {}, while the island grew by {}.{source}{}",
            fmt_thousands_f0(p0),
            fmt_thousands_f0(p1),
            fmt_pct(g, 0),
            fmt_pct(island, 0),
            known_for(st, C, t)
        ),
    );
    ev.town = Some(t);
    ev.cell = Some(st.town_node[t]);
    ev.why = why(
        P,
        "the decade's fastest-growing town",
        &format!("people in {name} and its district"),
        g,
        island + Q.fastest_margin,
        Some(island),
        format!("grew {} over the decade against the island's {} (it needs the island's growth and {} more)", fmt_pct(g, 0), fmt_pct(island, 0), fmt_pct(Q.fastest_margin, 0)),
    );
    events.push(ev);
}

/// A decade in which several towns were founded gets one item for them all.
/// The first decade counts from the first year recorded (`start_year`), so a
/// town the warm-up founded is never news.
fn new_towns_of_decade(st: &State, P: &Params, year: i64, start_year: i64, events: &mut Vec<Event>) {
    let decade = decade_of(year);
    let from = decade.max(start_year);
    let mut towns: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t] && (from..=year).contains(&st.town_founded[t])).collect();
    if towns.len() < P.chronicle.new_towns_min.max(1) {
        return;
    }
    towns.sort_by(|&a, &b| st.town_founded[a].cmp(&st.town_founded[b]).then(a.cmp(&b)));
    let largest = *towns.iter().max_by(|&&a, &&b| st.pop[a].total_cmp(&st.pop[b]).then(b.cmp(&a))).expect("some towns");
    let names: Vec<String> = towns.iter().map(|&t| format!("{} ({})", st.town_name[t], st.town_founded[t])).collect();
    let mut ev = Event::new(
        year as f64 + 0.9,
        "town",
        0.65,
        format!("{} new towns in the {decade}s", capital(&number(towns.len()))),
        format!(
            "{} grew up in the decade where people were working far from any market. {} is the largest, with {} people in its district.",
            and_list(&names),
            st.town_name[largest],
            fmt_thousands_f0(st.pop[largest])
        ),
    );
    ev.town = Some(largest);
    ev.cell = Some(st.town_node[largest]);
    ev.why = why(
        P,
        "towns founded in a decade",
        "towns",
        towns.len() as f64,
        P.chronicle.new_towns_min as f64,
        None,
        format!("{} towns were founded from {from} to {year} (it needs {})", towns.len(), P.chronicle.new_towns_min),
    );
    events.push(ev);
}

/// The year's hunger (`people::growth`) says how short the food ran. The 0.82
/// is the port's line for a famine.
pub fn famine_provenance(st: &State, P: &Params, year: i64, events: &mut [Event]) {
    if !P.chronicle.enabled {
        return;
    }
    for e in events.iter_mut().filter(|e| e.kind == "famine" && e.why.is_none() && e.year.floor() as i64 == year) {
        let Some(t) = e.town else { continue };
        let hit = (0..st.T_max).filter(|&i| st.town_active[i] && st.food_min_acc[i] < 0.82 && st.town_founded[i] < year - 2).count();
        e.why = why(
            P,
            "food short at the worst month of the year",
            &format!("the least share of their food that households in {} got in a month", st.town_name[t]),
            st.food_min_acc[t],
            0.82,
            None,
            format!("{} got {} of its food at the worst point (hunger is under 82%); {} went hungry, and three or more are told as one", st.town_name[t], fmt_pct(st.food_min_acc[t], 0), if hit == 1 { "one town".to_string() } else { format!("{} towns", number(hit)) }),
        );
    }
}

/// The year's large closures (`sim::notable_closures`) say how large.
fn closure_provenance(st: &State, C: &Content, P: &Params, year: i64, events: &mut [Event]) {
    for e in events.iter_mut().filter(|e| e.kind == "closure" && e.why.is_none() && e.year.floor() as i64 == year) {
        let Some(f) = e.firm else { continue };
        let K0 = C.m_K0[st.f_method[f]];
        e.why = why(
            P,
            "a large works closes",
            &format!("capacity of {}", st.f_name[f]),
            st.f_K[f],
            1.5 * K0,
            Some(K0),
            format!("it was {:.1} times the usual size of its kind (it needs 1.5); at most two closures a year are told", st.f_K[f] / K0),
        );
    }
}

// ---------------------------------------------------------------- ranking

/// The kinds that count as one when a decade's places are shared out.
fn class(e: &Event) -> &str {
    match e.kind.as_str() {
        "alarm" | "verdict" => "cycle",
        "railway" | "rail_plan" => "railway",
        // canals (docs/rules/canals.md): planned and opened are one kind too
        "canal" | "canal_plan" => "canal",
        // the harvest's own news: a dearth is the price of a short crop
        "harvest" | "dearth" => "harvest",
        "family" if e.family == Some(0) => "own",
        kind => kind,
    }
}

/// What an item is ranked by: its weight, except that a dated slump gains its
/// depth (a fifth at most), which its provenance carries. The observer weighs a
/// slump of 5 per cent and a depression of 20 the same.
fn standing(e: &Event) -> f64 {
    if e.kind == "verdict" {
        if let Some(Why { threshold: peak, baseline: Some(trough), .. }) = &e.why {
            if *peak > 0.0 {
                return e.weight + (1.0 - trough / peak).clamp(0.0, 0.2);
            }
        }
    }
    e.weight
}

/// Mark the items a history would keep (`Event::kept`), decade by decade.
///
/// Within a decade the heaviest go first, with at most `per_kind` of a kind
/// (`inventions` for discoveries) and `per_decade` in all, and nothing lighter
/// than `floor`. One place is the player's: the heaviest of the family's own
/// items at `floor` or above is kept whatever else happened. A decade that is
/// over and still has fewer than `at_least` takes the best of what is left
/// above `fill_floor`. An alarm gives way to the verdict that later dates its
/// slump. Ties go to the earlier item.
///
/// `upto` is the last year simulated: a decade is over when its last year is.
/// The whole list is ranked afresh each time, so a live game's marks at the
/// end are those `sim::finish` gives the recorded run.
pub fn rank(events: &mut [Event], P: &Params, start_year: i64, end_year: i64, upto: i64) {
    let Q = &P.chronicle;
    if !Q.enabled {
        return;
    }
    for e in events.iter_mut() {
        e.kept = Some(false);
    }
    // decades by the calendar (a run from 1755 has a short first one), counted
    // by their first year over ten; anything before the start goes with the
    // first, and the last takes in a final odd year (1900 goes with the 1890s)
    let first = start_year.div_euclid(10);
    let last = (end_year - 1).div_euclid(10).max(first);
    let decade = |e: &Event| (e.year.floor() as i64).div_euclid(10).clamp(first, last);
    // alarms that a verdict has since dated
    let mut dated = vec![false; events.len()];
    for v in events.iter().filter(|e| e.kind == "verdict") {
        let Some(span) = v.span else { continue };
        for (i, a) in events.iter().enumerate() {
            if a.kind == "alarm" && a.year >= span[0].floor() && a.year <= v.year {
                dated[i] = true;
            }
        }
    }
    let mut by_decade: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
    for (i, e) in events.iter().enumerate() {
        if !dated[i] && standing(e) >= Q.fill_floor {
            by_decade.entry(decade(e)).or_default().push(i);
        }
    }
    for (d, mut items) in by_decade {
        items.sort_by(|&a, &b| standing(&events[b]).total_cmp(&standing(&events[a])).then(events[a].year.total_cmp(&events[b].year)).then(a.cmp(&b)));
        let over = upto >= if d == last { end_year } else { 10 * d + 9 };
        let mut kept: Vec<usize> = Vec::new();
        let mut of_class: BTreeMap<String, usize> = BTreeMap::new();
        let mut keep = |i: usize, kept: &mut Vec<usize>, events: &[Event]| -> bool {
            let c = class(&events[i]);
            let cap = if c == "discovery" { Q.inventions } else { Q.per_kind };
            let n = of_class.entry(c.to_string()).or_insert(0);
            if kept.len() >= Q.per_decade || *n >= cap || kept.contains(&i) {
                return false;
            }
            *n += 1;
            kept.push(i);
            true
        };
        // the player's own great moment has its place
        if let Some(&own) = items.iter().find(|&&i| class(&events[i]) == "own" && standing(&events[i]) >= Q.floor) {
            keep(own, &mut kept, events);
        }
        for &i in items.iter().filter(|&&i| standing(&events[i]) >= Q.floor) {
            keep(i, &mut kept, events);
        }
        if over {
            for &i in items.iter().filter(|&&i| standing(&events[i]) < Q.floor) {
                if kept.len() >= Q.at_least {
                    break;
                }
                keep(i, &mut kept, events);
            }
        }
        for i in kept {
            events[i].kept = Some(true);
        }
    }
}

// ---------------------------------------------------------------- words

/// "three", or the figure past twelve.
fn number(n: usize) -> String {
    const WORDS: [&str; 13] = ["no", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve"];
    WORDS.get(n).map_or_else(|| n.to_string(), |w| w.to_string())
}

/// "third", or "11th", "21st", "22nd" past tenth.
fn ordinal(n: usize) -> String {
    const WORDS: [&str; 11] = ["", "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth"];
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    WORDS.get(n).filter(|w| !w.is_empty()).map_or_else(|| format!("{n}{suffix}"), |w| w.to_string())
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// "a", "a and b", "a, b and c".
fn and_list(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        n => format!("{} and {}", items[..n - 1].join(", "), items[n - 1]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::railways::CoalLine;

    #[test]
    fn a_coal_boom_says_the_railway_reached_a_town_only_of_its_first_line() {
        let towns: Vec<String> = ["Brambridge", "Garside", "Sandmere", "Glenbrook"].iter().map(|s| s.to_string()).collect();
        let line = |opened: f64, from: Option<usize>, nth: usize, first: f64| Some(CoalLine { opened, colliery: false, from, nth, first });
        let branch = "A branch line to the collieries";
        // the town's first line, since the year the boom looks back to
        assert_eq!(carried_by("Garside", 1827, line(1827.6, Some(0), 1, 1827.6), branch, &towns), " The railway that reached Garside in 1827 carries it away.");
        // island 7's Garside: the line from Brambridge came in 1827, the one from Sandmere in 1829
        assert_eq!(
            carried_by("Garside", 1827, line(1829.3, Some(2), 2, 1827.6), branch, &towns),
            " The railway reached Garside in 1827, and a second line, from Sandmere, in 1829; they carry it away."
        );
        // island 7's Stratbank: its first line came in 1823, long before the boom's base year
        assert_eq!(carried_by("Stratbank", 1833, line(1835.2, Some(3), 2, 1823.5), branch, &towns), " The line from Glenbrook, opened in 1835, carries it away.");
        assert_eq!(carried_by("Stratbank", 1833, line(1835.2, None, 3, 1823.5), branch, &towns), " A third line to Stratbank, opened in 1835, carries it away.");
        // a branch to the pits, and nothing at all
        let pits = Some(CoalLine { opened: 1838.9, colliery: true, from: None, nth: 0, first: 1838.9 });
        assert_eq!(carried_by("Garside", 1835, pits, branch, &towns), " A branch line to the collieries, opened in 1838, brings it down from the pits.");
        assert_eq!(carried_by("Garside", 1835, None, branch, &towns), "");
    }
}
