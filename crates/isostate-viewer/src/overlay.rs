//! Map overlays: the numbers behind the picture, washed over the ground so the
//! map can answer "why is that there". Land rent, each town's market area, how
//! far every square is from its market, what a good costs across the island,
//! where a good is made, what a month's work pays, the towns' ground, and what
//! the ground holds.
//!
//! Each is one tint per square of the simulation's grid, in a few muted inks,
//! written to a texture the ground shader reads (terrain.wesl, `overlay_wash`).
//! A legend card says what is shown, with round figures along the scale, and
//! the inspector says the value in the square that was clicked. Nothing is
//! shown that the simulation does not record. Where the clock stands in a
//! month of a live game, the price and wage maps show that month's figures
//! (the last month that is over), which the viewer has kept since the game was
//! opened (`live::Months`); everywhere else they show the year's average, and
//! the rest of the views are the year's, as the map is.
//!
//! Where a view colours by a number, its palest and darkest tints stand at
//! ends taken one of three ways, this year's, thirty years' gliding, or the
//! whole run's (`ranges.rs`), chosen on the legend card for each view. A
//! year's ends are reckoned once and kept (`EndsCache`), off the main thread
//! when many years want them at once.
//!
//! Five of the views (residents, wages, rent, prices, made here) have a Trend
//! switch on the card: coloured by how their figure changed over the last 5,
//! 10 or 30 years, with the towns that rose and fell fastest named under the
//! legend. The reckoning is `trend.rs`, apart from the painting here.

// see the note at the top of ui.rs
#![allow(deprecated)]

#[path = "trend.rs"]
pub mod trend;

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::tasks::Task;
use bevy::ui::FocusPolicy;

use crate::family::{chip_button, FamilyView};
use crate::live::Months;
use crate::ranges::{self, Ends, Range};
use crate::replay::{self, Replay};
use crate::ui::{self, text, INK, MUTED, RULE};
use crate::{terrain, Data, SeasonMats, Timeline, UiFont};
use trend::Span;

/// How much of the ground's own colour the wash replaces.
const STRENGTH: f32 = 0.88;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Overlay {
    Rent,
    Areas,
    Reach,
    Price,
    /// how much of a good was made on each square or in each town
    Made,
    Wages,
    /// (task A) when each square of town ground joined its town
    Growth,
    /// (task A) the people living on each square of town ground
    Residents,
    /// what the ground holds: the deposits, the standing timber, the soil
    Deposits,
    /// what each way between two squares carried, drawn as ribbons (`traffic.rs`)
    Traffic,
}

impl Overlay {
    pub const ALL: [Overlay; 10] = [Overlay::Rent, Overlay::Areas, Overlay::Reach, Overlay::Price, Overlay::Made, Overlay::Wages, Overlay::Growth, Overlay::Residents, Overlay::Deposits, Overlay::Traffic];

    /// The word on its chip.
    pub fn chip(self) -> &'static str {
        match self {
            Overlay::Rent => "Rent",
            Overlay::Areas => "Markets",
            Overlay::Reach => "Reach",
            Overlay::Price => "Prices",
            Overlay::Made => "Made here",
            Overlay::Wages => "Wages",
            Overlay::Growth => "Town growth",
            Overlay::Residents => "Residents",
            Overlay::Deposits => "Deposits",
            Overlay::Traffic => "Traffic",
        }
    }

    /// Its word on the command line and in scripts.
    pub fn word(self) -> &'static str {
        match self {
            Overlay::Rent => "rent",
            Overlay::Areas => "areas",
            Overlay::Reach => "reach",
            Overlay::Price => "price",
            Overlay::Made => "made",
            Overlay::Wages => "wages",
            Overlay::Growth => "growth",
            Overlay::Residents => "residents",
            Overlay::Deposits => "deposits",
            Overlay::Traffic => "traffic",
        }
    }

    /// Whether it colours by a number, and so has a range to choose.
    pub fn scaled(self) -> bool {
        self != Overlay::Areas
    }

    /// The range it opens with: thirty years, except where the view has a
    /// scale of its own nature (when town ground joined, the run's years; how
    /// rich the ground is, from nothing to the richest there was).
    fn default_range(self) -> Range {
        match self {
            Overlay::Growth | Overlay::Deposits => Range::Whole,
            _ => Range::Thirty,
        }
    }

    /// How its figures lie on its tints: by ratio (rent, residents, what is
    /// made, which run over powers of ten) or evenly, and the low end where
    /// the view's nature sets it.
    fn spec(self) -> Spec {
        match self {
            Overlay::Rent | Overlay::Residents | Overlay::Made | Overlay::Traffic => Spec { log: true, pinned: None },
            Overlay::Reach | Overlay::Deposits => Spec { log: false, pinned: Some(0.0) },
            _ => Spec { log: false, pinned: None },
        }
    }
}

#[derive(Clone, Copy)]
struct Spec {
    log: bool,
    pinned: Option<f32>,
}

/// A figure in its scale's measure (`ranges.rs`): its logarithm on a scale by ratio.
fn measure(v: f32, log: bool) -> f32 {
    if log { v.max(1e-9).ln() } else { v }
}

fn unmeasure(m: f32, log: bool) -> f32 {
    if log { m.exp() } else { m }
}

// ------------------------------------------------------------------ what the ground holds

/// One kind of thing the ground holds, as the deposits view steps through them.
pub struct Kind {
    pub word: &'static str,
    pub label: &'static str,
    /// the ground's field it reads (`replay::Holds::fields`), or "cover" for the standing timber
    field: &'static str,
    stops: [u32; 5],
}

/// Coal in soot, iron ore in rust, clay in terracotta, stone in slate grey,
/// timber in pine (a blue-green, not the fields' green), the soil in umber and
/// the grazing in straw.
pub const KINDS: [Kind; 7] = [
    Kind { word: "coal", label: "Coal", field: "coal", stops: [0xf0eee9, 0xcfc9c0, 0x9d958c, 0x64605c, 0x2e2c2c] },
    Kind { word: "ore", label: "Iron ore", field: "ore", stops: [0xf5e9e0, 0xe7c3a8, 0xcc8f6a, 0xa5603f, 0x6e3a25] },
    Kind { word: "clay", label: "Clay", field: "clay", stops: [0xf6ebe3, 0xecc7b0, 0xd99c7c, 0xb86f50, 0x834633] },
    Kind { word: "stone", label: "Stone", field: "rock", stops: [0xeeeeea, 0xd3d6d6, 0xaeb4b8, 0x828b93, 0x545d68] },
    Kind { word: "timber", label: "Standing timber", field: "cover", stops: [0xebf0ea, 0xc3d7cf, 0x8fb5aa, 0x5a8a80, 0x2f5c56] },
    Kind { word: "soil", label: "Soil", field: "fertility", stops: [0xf4ecd8, 0xe2cfa2, 0xc7a56a, 0x9b773f, 0x664a24] },
    Kind { word: "grazing", label: "Grazing", field: "pasture", stops: [0xf3f0dc, 0xe2ddae, 0xc7bf79, 0xa19a4f, 0x6d6a33] },
];

fn find_kind(t: &str) -> Option<usize> {
    KINDS.iter().position(|k| k.word.eq_ignore_ascii_case(t) || k.label.eq_ignore_ascii_case(t) || k.field.eq_ignore_ascii_case(t))
}

// ------------------------------------------------------------------ naming a view

/// A map view as the command line, a `--view` or a script names it: the view
/// (None for the plain map), the good or the kind of ground it shows if one
/// is named, the range if one is named, and the trend if one is named
/// (Some(None) for "+trendoff").
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Choice {
    pub kind: Option<Overlay>,
    pub which: Option<usize>,
    pub range: Option<Range>,
    pub trend: Option<Option<Span>>,
}

/// The parts after a view's name: "+year", "+30" or "+all" for its range,
/// "+trend5", "+trend10", "+trend30" or "+trendoff" for its trend, in any
/// order. None if one is neither.
fn suffixes<'a>(parts: impl Iterator<Item = &'a str>) -> Option<(Option<Range>, Option<Option<Span>>)> {
    let (mut range, mut trend) = (None, None);
    for p in parts {
        if let Some(t) = trend::parse_word(p) {
            trend = Some(t);
        } else {
            range = Some(Range::parse(p)?);
        }
    }
    Some((range, trend))
}

/// For the command line: "rent", "areas", "reach", "price", "price:coal",
/// "made:grain", "wages", "growth", "residents", "deposits:coal", or "plain"
/// for no overlay, each with "+year", "+30" or "+all" after it for its range
/// and "+trend10" (5, 10, 30 or off) for its trend. The good is by name or
/// label, and stays as it was if none is given. None if the word is not an
/// overlay at all.
pub fn parse(word: &str, r: &Replay) -> Option<Choice> {
    let w = word.to_ascii_lowercase();
    let mut parts = w.split('+');
    let w = parts.next().unwrap_or("");
    let (range, trend) = suffixes(parts)?;
    let (head, tail) = w.split_once(':').map_or((w, None), |(a, b)| (a, Some(b)));
    let kind = if head == "plain" { None } else { Some(Overlay::ALL.into_iter().find(|o| o.word() == head)?) };
    let which = match kind {
        Some(Overlay::Deposits) => tail.and_then(find_kind),
        _ => tail.and_then(|t| find_good(&r.header.goods, t)),
    };
    Some(Choice { kind, which, range, trend })
}

/// The good a word names, by its name or its label, whatever its case.
fn find_good(goods: &[replay::Good], t: &str) -> Option<usize> {
    goods.iter().position(|g| g.name.eq_ignore_ascii_case(t) || g.label.eq_ignore_ascii_case(t))
}

/// What is wrong with a word given for an overlay on the command line, if
/// anything: a name that is no overlay, or a good the run does not have. Such
/// a word is otherwise passed over (the plain map, or the good as it was).
pub fn complaint(word: &str, r: &Replay) -> Option<String> {
    complaint_among(word, &r.header.goods)
}

/// `complaint`, for a run whose goods are these: a game's are known from the
/// rules it is played under before its island is made.
pub fn complaint_among(word: &str, goods: &[replay::Good]) -> Option<String> {
    if !is_word(word) {
        return Some(format!("\"{word}\" is not a map view: {}", VIEWS_WORDS));
    }
    let w = word.split('+').next().unwrap_or("");
    let (head, tail) = w.split_once(':')?;
    if head.eq_ignore_ascii_case("deposits") {
        if find_kind(tail).is_some() {
            return None;
        }
        let names: Vec<&str> = KINDS.iter().map(|k| k.word).collect();
        return Some(format!("\"{tail}\" is not a kind of ground: use one of {}", names.join(", ")));
    }
    if find_good(goods, tail).is_some() {
        return None;
    }
    let names: Vec<&str> = goods.iter().map(|g| g.name.as_str()).collect();
    Some(format!("\"{tail}\" is not a good of this run: use one of {}", names.join(", ")))
}

/// The words a map view takes, for the messages that refuse one.
pub const VIEWS_WORDS: &str = "use rent, areas, reach, price, price:GOOD, made, made:GOOD, wages, growth, residents, deposits, deposits:KIND, traffic or plain, each with +year, +30 or +all after it for its range if wanted, and +trend5, +trend10, +trend30 or +trendoff for its trend";

/// Whether a word of a `--view` names an overlay (before the run is loaded).
pub fn is_word(word: &str) -> bool {
    let w = word.to_ascii_lowercase();
    let mut parts = w.split('+');
    let w = parts.next().unwrap_or("");
    if suffixes(parts).is_none() {
        return false;
    }
    let head = w.split(':').next().unwrap_or("");
    head == "plain" || Overlay::ALL.iter().any(|o| o.word() == head)
}

// ------------------------------------------------------------------ the resource

/// Each year's own ends of each view, reckoned once (`ranges::year_ends`), and
/// the reckoning of many years at once while it goes on off the main thread.
#[derive(Default)]
struct EndsCache {
    /// by view, the good or kind it shows, and its trend if it shows one: for
    /// each year, its ends once reckoned
    years: HashMap<EndsKey, Vec<Option<Option<Ends>>>>,
    /// the years being reckoned off the main thread, for one view
    task: Option<(EndsKey, Task<Vec<(usize, Option<Ends>)>>)>,
    /// raised when a reckoning off the main thread lands
    version: u64,
    /// how many years were reckoned on the main thread, and off it (for the notes)
    pub inline: u64,
    pub away: u64,
}

/// A view's ends are kept by the view, its good or kind of ground, and its trend.
type EndsKey = (Overlay, usize, Option<Span>);

/// At most this many years' ends are reckoned in the frame; more go off the main thread.
const INLINE_YEARS: usize = 3;

/// Year `k`'s own ends of a view, or of its trend over `span`.
pub fn ends_of(r: &Replay, k: usize, kind: Overlay, which: usize, span: Option<Span>) -> Option<Ends> {
    match span {
        Some(s) => trend::trend_ends(r, k, kind, which, s),
        None => year_ends(r, k, kind, which),
    }
}

/// What is washed over the map, if anything.
#[derive(Resource)]
pub struct MapOverlay {
    pub kind: Option<Overlay>,
    /// the good whose price, or where it is made, the overlay shows
    pub good: usize,
    /// the kind of ground the deposits view shows (`KINDS`)
    pub deposit: usize,
    /// the overlay the pill brings back
    last: Overlay,
    /// the range chosen on each view's card, while the viewer is open
    ranges: HashMap<Overlay, Range>,
    /// the Trend switch: one setting for every view that has a trend
    /// (`trend::trends`), kept while the others show the present
    pub trend: Option<Span>,
    /// what the texture holds: overlay, good or kind, year index, years filled,
    /// the month whose prices or wages it shows in place of the year's, the
    /// range, the trend, and the version of the ends it was painted with
    painted: Option<(Option<Overlay>, usize, usize, usize, Option<usize>, Option<Range>, Option<Span>, u64)>,
    legend: Option<Legend>,
    /// bumped when the legend card must be rebuilt
    version: u64,
    card: Option<u64>,
    ends: EndsCache,
    /// the scale the view was last painted on, where it colours by a number:
    /// the traffic's ribbons are laid on it (`traffic.rs`)
    pub scale: Option<Scale>,
    /// GROUND-BRUSH: the brush's strokes of the year in progress, painted over
    /// the Deposits view at once (ground.rs) until the year's end brings them
    pub preview: Preview,
}

/// What the brush has painted since the last year came, until the next one
/// brings the island's own figures for it: the year (as an index) they are
/// laid over and every year after it, and for each square a kind (`KINDS`)
/// and what it holds.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Preview {
    pub from: usize,
    pub cells: Vec<(usize, usize, f32)>,
}

impl MapOverlay {
    /// The view shown, as `--overlay` takes it ("rent", "price:coal",
    /// "deposits:clay"), or None for the plain map (for a note: note.rs).
    pub fn word(&self, r: &Replay) -> Option<String> {
        let kind = self.kind?;
        Some(match kind {
            Overlay::Price | Overlay::Made => format!("{}:{}", kind.word(), r.header.goods.get(self.good).map_or("", |g| g.name.as_str())),
            Overlay::Deposits => format!("deposits:{}", KINDS.get(self.deposit).map_or("coal", |k| k.word)),
            _ => kind.word().to_string(),
        })
    }

    /// An edit taken back (back.rs): the ends reckoned for the years from
    /// `filled` on are forgotten, a reckoning on its way (of the run that is
    /// gone) is let go, and the map is painted again.
    pub fn forget(&mut self, filled: usize) {
        for years in self.ends.years.values_mut() {
            years.truncate(filled);
        }
        self.ends.task = None;
        self.ends.version += 1;
        self.painted = None;
        self.version += 1;
        self.preview = Preview::default();
    }

    pub fn new(kind: Option<Overlay>, good: usize) -> MapOverlay {
        MapOverlay { kind, good, deposit: 0, last: kind.unwrap_or(Overlay::Rent), ranges: HashMap::new(), trend: None, painted: None, legend: None, version: 0, card: None, ends: EndsCache::default(), scale: None, preview: Preview::default() }
    }

    /// The town in place `i` among the movers the card shows: 0 to 2 the
    /// rising, 3 to 5 the falling (for a script's `mover:`).
    pub fn mover(&self, i: usize) -> Option<usize> {
        let (up, down) = self.legend.as_ref()?.movers.as_ref()?;
        if i < trend::MOVERS { up.get(i) } else { down.get(i - trend::MOVERS) }.map(|m| m.town)
    }

    /// The trend the view on shows, if the switch is on and the view has one.
    pub fn trend_shown(&self) -> Option<Span> {
        self.trend.filter(|_| self.kind.is_some_and(trend::trends))
    }

    /// GROUND-BRUSH: the brush's preview put in place, and the map painted again.
    pub fn set_preview(&mut self, p: Preview) {
        if self.preview != p {
            self.preview = p;
            self.painted = None;
        }
    }

    /// The view a `Choice` names, with its good, kind of ground and range.
    pub fn choose(&mut self, c: &Choice) {
        self.set(c.kind);
        if let (Some(kind), Some(w)) = (c.kind, c.which) {
            if kind == Overlay::Deposits {
                self.deposit = w.min(KINDS.len() - 1);
            } else {
                self.good = w;
            }
        }
        if let (Some(kind), Some(range)) = (c.kind, c.range) {
            self.ranges.insert(kind, range);
        }
        if let Some(t) = c.trend {
            self.trend = t;
        }
    }

    pub fn set(&mut self, kind: Option<Overlay>) {
        self.kind = kind;
        if let Some(k) = kind {
            self.last = k;
        }
    }

    /// The range a view's colours are spread over now.
    pub fn range(&self, kind: Overlay) -> Range {
        self.ranges.get(&kind).copied().unwrap_or(kind.default_range())
    }

    /// The good or the kind of ground a view shows (nought for the others).
    pub fn which(&self, kind: Overlay) -> usize {
        match kind {
            Overlay::Price | Overlay::Made => self.good,
            Overlay::Deposits => self.deposit,
            _ => 0,
        }
    }

    /// The overlay after this one, round through "none"; `back` goes the other way.
    fn step(&mut self, back: bool) {
        let all: Vec<Option<Overlay>> = std::iter::once(None).chain(Overlay::ALL.into_iter().map(Some)).collect();
        let i = all.iter().position(|&o| o == self.kind).unwrap_or(0);
        let n = all.len();
        self.set(all[if back { (i + n - 1) % n } else { (i + 1) % n }]);
    }

    /// The overlay goes by market towns, so the map should name them all.
    pub fn names_towns(&self) -> bool {
        matches!(self.kind, Some(Overlay::Areas | Overlay::Price | Overlay::Made | Overlay::Wages | Overlay::Growth | Overlay::Residents | Overlay::Traffic))
    }

    /// The month whose prices or wages the map shows in place of the year's, if it shows one.
    pub fn month(&self) -> Option<usize> {
        self.painted.and_then(|p| p.4)
    }

    /// What the overlay says of a square, for the inspector: of the month the map shows, if it shows one.
    pub fn said(&self, r: &Replay, months: &Months, k: usize, c: usize) -> Option<String> {
        let month = self.painted.and_then(|p| p.4).map(|t| Month::of(r, months, t));
        let kind = self.kind?;
        let now = said(r, k, c, kind, self.which(kind), month.as_ref());
        // under a trend, what became of the square, or of its town, over the span
        match (now, self.trend_shown().and_then(|s| trend::trend_map(r, k, kind, self.which(kind), s))) {
            (Some(now), Some(m)) if c < r.n => {
                let y0 = r.y0 + m.from as i32;
                let whose = if m.by_town { market(r, k, c).map_or(String::new(), |t| format!(", {}'s", town_name(r, t))) } else { String::new() };
                let change = match m.cells[c] {
                    trend::Change::Rate(v) => format!(" Since {y0}{whose}: {} a year.", trend::rate_words(v)),
                    trend::Change::New => format!(" Since {y0}{whose}: new, with nothing there in {y0}."),
                    trend::Change::Gone => format!(" Since {y0}{whose}: gone, with something there in {y0} and nothing now."),
                    trend::Change::Nothing => String::new(),
                };
                Some(now + &change)
            }
            (now, _) => now,
        }
    }

    /// For a script's `map` step: the view's range and its ends as painted, in
    /// words, with the least and the greatest of what year `k` shows.
    pub fn range_said(&self, r: &Replay, k: usize) -> String {
        match (self.kind, &self.legend) {
            (Some(_), Some(l)) if self.trend_shown().is_some() && l.range.is_some() => {
                // the trend: its span, its ends in per cent a year, and the movers with their rates
                let names = |m: &[trend::Mover]| m.iter().map(|m| format!("{} {}", m.name, trend::rate_words(m.rate))).collect::<Vec<_>>().join(", ");
                let (up, down) = l.movers.as_ref().map_or((String::new(), String::new()), |(a, b)| (names(a), names(b)));
                format!(
                    "trend: {} years range: {} ({}) ticks: {} keys: {} rising: [{up}] falling: [{down}]",
                    self.trend_shown().map_or(0, |s| s.years()),
                    self.range(self.kind.unwrap()).word(),
                    l.ends,
                    l.ticks.iter().map(|t| t.1.as_str()).collect::<Vec<_>>().join(" "),
                    l.keys.iter().map(|k| k.1.as_str()).collect::<Vec<_>>().join(", ")
                )
            }
            (Some(kind), Some(l)) if kind.scaled() && l.range.is_some() => {
                let extremes = sample(r, k, kind, self.which(kind)).map(|v| v.iter().fold((f32::MAX, f32::MIN), |a, &x| (a.0.min(x), a.1.max(x))));
                let ext = extremes.map_or(String::new(), |(a, b)| format!(" the year's least and greatest: {} to {}", tick_words(kind, a), tick_words(kind, b)));
                format!("range: {} ({}){ext} ticks: {}", self.range(kind).word(), l.ends, l.ticks.iter().map(|t| t.1.as_str()).collect::<Vec<_>>().join(" "))
            }
            _ => "range: none".to_string(),
        }
    }
}

/// A month whose prices and wages a map shows in place of the year's averages.
pub struct Month<'a> {
    pub months: &'a Months,
    /// the month's place in the run (0 is the first January)
    pub t: usize,
    /// "March 1784"
    pub label: String,
}

impl Month<'_> {
    pub fn of<'a>(r: &Replay, months: &'a Months, t: usize) -> Month<'a> {
        Month { months, t, label: format!("{} {}", crate::clock::MONTHS[t % 12], r.y0 + (t / 12) as i32) }
    }
}

/// The month the price and wage maps show for a clock at `t`: the last month
/// that is over, if the viewer holds it or an earlier one of the same year.
/// None where the date shows no month, and where nothing is held (a recorded
/// run; a year a game played before it was opened).
pub fn month_shown(months: &Months, t: f32, dated: bool) -> Option<usize> {
    if !dated {
        return None;
    }
    months.at_or_before(crate::clock::series_month(t).checked_sub(1)?)
}

// ------------------------------------------------------------------ the tints

/// Five stops from low to high, in sRGB. Inks that sit on the paper: sepia for
/// rent, slate for distance, madder for prices, mulberry for wages, ochre for
/// what is made. None is the green of the fields, so a map view never reads as
/// the land itself.
const SEPIA: [u32; 5] = [0xf4ead0, 0xe6cf9c, 0xd2aa6a, 0xb07e45, 0x80552e];
const SLATE: [u32; 5] = [0xeeeee2, 0xc9d6d2, 0x9bb3ba, 0x6d8d9e, 0x45647a];
const MADDER: [u32; 5] = [0xf5eadc, 0xe8cfbb, 0xd2aa96, 0xb28171, 0x86574d];
const MULBERRY: [u32; 5] = [0xf2ebe6, 0xdccbd2, 0xbfa3b8, 0x9a7b9b, 0x6f557c];
const OCHRE: [u32; 5] = [0xf7efd2, 0xf0d98f, 0xe0b34f, 0xb98227, 0x7d5417];
/// (task A) Town ground by the year it joined, from the oldest (dark brick)
/// to the newest (straw), and its residents from few (pale) to many (indigo).
const BRICK: [u32; 5] = [0x5e2f2b, 0x9c4a3a, 0xcf7b4c, 0xe6ae5e, 0xf2d98a];
const INDIGO: [u32; 5] = [0xc9d4ee, 0xaebde0, 0x8497c9, 0x5a6dab, 0x333f7d];
/// (task A) The low end of the scale of when town ground joined: the middles
/// of the first towns are dated a century before the record opens.
const OLDEST_GROUND: f32 = 1650.0;
/// Market areas: six soft tints, so that no two neighbours share one. Straw,
/// sage, clay, slate, heather and lichen: washes, not flags.
const AREAS: [u32; 6] = [0xe8d3a6, 0xb9cdb0, 0xdcb5a2, 0xb3c3cf, 0xc8b8c6, 0xd3d29a];
/// Land that trades in no market, or that the year has no figure for.
const BLANK: u32 = 0xd9d3c4;

fn rgb(h: u32) -> [u8; 3] {
    [(h >> 16) as u8, (h >> 8) as u8, h as u8]
}

/// The tint at `t` from 0 (low) to 1 (high) along five stops.
fn ramp(stops: &[u32; 5], t: f32) -> [u8; 3] {
    let x = t.clamp(0.0, 1.0) * 4.0;
    let i = (x.floor() as usize).min(3);
    let f = x - i as f32;
    let (a, b) = (rgb(stops[i]), rgb(stops[i + 1]));
    [0, 1, 2].map(|j| (a[j] as f32 + (b[j] as f32 - a[j] as f32) * f).round() as u8)
}

// ------------------------------------------------------------------ the figures

/// A scale between two figures: where a value falls from 0 (at or below the
/// low end) to 1 (at or above the high end), evenly or by ratio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scale {
    pub lo: f32,
    pub hi: f32,
    pub log: bool,
}

impl Scale {
    pub fn at(&self, v: f32) -> f32 {
        if !(self.hi > self.lo) {
            return 0.5;
        }
        let t = if self.log { (v.max(1e-9) / self.lo).ln() / (self.hi / self.lo).ln() } else { (v - self.lo) / (self.hi - self.lo) };
        t.clamp(0.0, 1.0)
    }

    fn of(e: Ends, log: bool) -> Scale {
        Scale { lo: unmeasure(e.lo, log), hi: unmeasure(e.hi, log), log }
    }
}

/// `v` to two significant figures.
pub fn round_sig(v: f32) -> f32 {
    if v <= 0.0 || !v.is_finite() {
        return 0.0;
    }
    let mag = 10f32.powf(v.log10().floor() - 1.0);
    (v / mag).round() * mag
}

/// Money, small sums to the penny: prices and wages are near one.
pub fn coin(v: f32) -> String {
    if v >= 9999.5 {
        ui::money(v)
    } else if v >= 99.95 {
        format!("\u{00a3}{}", ui::thousands(v))
    } else if v >= 9.995 {
        format!("\u{00a3}{v:.1}")
    } else {
        format!("\u{00a3}{v:.2}")
    }
}

/// A sum that is only known roughly, to two figures and without pence it does
/// not have: £97 where `coin` would write £97.0.
fn about(v: f32) -> String {
    let v = round_sig(v);
    if (9.995..99.95).contains(&v) { format!("\u{00a3}{v:.0}") } else { coin(v) }
}

/// An amount of a good, in its own units: 0.4, 12, 3,400, 1.2m.
fn amount(v: f32) -> String {
    if v >= 999_500.0 {
        format!("{:.1}m", v / 1e6)
    } else if v >= 99.5 {
        ui::thousands(v)
    } else if v >= 9.95 {
        format!("{v:.0}")
    } else if v >= 0.995 {
        format!("{v:.1}").trim_end_matches(".0").to_string()
    } else {
        format!("{v:.2}")
    }
}

/// A round figure on a legend's scale, as the view writes its figures.
fn tick_words(kind: Overlay, v: f32) -> String {
    match kind {
        Overlay::Rent => about(v),
        Overlay::Reach => format!("{v:.0} km"),
        Overlay::Price | Overlay::Wages => if v.fract().abs() < 1e-3 && v >= 1.0 { format!("\u{00a3}{v:.0}") } else { coin(v) },
        Overlay::Made => amount(v),
        Overlay::Growth => format!("{v:.0}"),
        Overlay::Residents => ui::thousands(v),
        Overlay::Deposits => {
            let s = format!("{v:.2}");
            let s = s.trim_end_matches('0');
            s.strip_suffix('.').map_or(s.to_string(), |w| w.to_string())
        }
        Overlay::Areas => String::new(),
        Overlay::Traffic => amount(v),
    }
}

fn land(r: &Replay, c: usize) -> bool {
    r.terrain[c] > 0
}

/// The town whose market a square trades in.
fn market(r: &Replay, k: usize, c: usize) -> Option<usize> {
    (r.catch[k * r.n + c] as usize).checked_sub(1).filter(|&t| t < r.header.towns.len())
}

fn town_name(r: &Replay, t: usize) -> &str {
    r.header.towns.get(t).map_or("a town", |t| t.name.as_str())
}

/// The rent a square's ground earns in a year (the record holds a month's).
fn rent_year(r: &Replay, k: usize, c: usize) -> Option<f32> {
    r.rent.get(k * r.n + c).map(|&b| 12.0 * replay::rent_of(b) * r.rent_scale(k))
}

/// How far a square is from its market, in kilometres of flat open country:
/// the record's path cost counts one for a 2 km square of lowland with no road,
/// in quarters. None where there is no way to a market at all.
fn reach_km(r: &Replay, k: usize, c: usize) -> Option<f32> {
    r.dist.get(k * r.n + c).filter(|&&b| b < 255).map(|&b| b as f32 * 0.5)
}

/// Whether any of a good has been made by the month a map shows, or by the
/// end of the year it shows (`replay::made_by`).
fn made(r: &Replay, k: usize, good: usize, month: Option<&Month>) -> bool {
    r.header.goods.get(good).is_none_or(|g| replay::made_by(r, &g.name, month.map_or(k * 12 + 11, |m| m.t)))
}

/// The year's mean price of a good in a town, if any was sold there.
fn price(r: &Replay, k: usize, t: usize, good: usize) -> Option<f32> {
    let ng = r.header.goods.len();
    r.town_price.get((k * r.n_towns + t) * ng + good).copied().filter(|p| p.is_finite() && *p > 0.0)
}

fn wage(r: &Replay, k: usize, t: usize) -> Option<f32> {
    r.town_wage.get(k * r.n_towns + t).copied().filter(|w| w.is_finite() && *w > 0.0)
}

/// The towns with a market area this year.
fn markets(r: &Replay, k: usize) -> Vec<usize> {
    let mut seen = vec![false; r.header.towns.len()];
    for c in 0..r.n {
        if land(r, c) {
            if let Some(t) = market(r, k, c) {
                seen[t] = true;
            }
        }
    }
    (0..seen.len()).filter(|&t| seen[t]).collect()
}

/// A tint for every market town so that no two towns whose areas touch share
/// one. Towns are taken in order and keep the tint of their number where they
/// can, so a town changes colour only when an older neighbour forces it to.
fn area_tints(r: &Replay, k: usize) -> Vec<usize> {
    let nt = r.header.towns.len();
    let mut touch = vec![false; nt * nt];
    for c in 0..r.n {
        let Some(a) = market(r, k, c).filter(|_| land(r, c)) else { continue };
        for d in [1, r.w] {
            let e = c + d;
            if e >= r.n || (d == 1 && e % r.w == 0) || !land(r, e) {
                continue;
            }
            if let Some(b) = market(r, k, e).filter(|&b| b != a) {
                touch[a * nt + b] = true;
                touch[b * nt + a] = true;
            }
        }
    }
    let mut tint = vec![0usize; nt];
    for t in 0..nt {
        let taken: Vec<usize> = (0..t).filter(|&u| touch[t * nt + u]).map(|u| tint[u]).collect();
        tint[t] = (0..AREAS.len()).map(|i| (t + i) % AREAS.len()).find(|i| !taken.contains(i)).unwrap_or(t % AREAS.len());
    }
    tint
}

/// Whether a square is a town's own ground in year `k`: kept town ground, or
/// a square the record calls town or works.
fn town_ground(r: &Replay, k: usize, c: usize) -> bool {
    r.joined(k, c).is_some() || matches!(r.use_.get(k * r.n + c), Some(9 | 10))
}

/// The share of a square a display class takes, as the record keeps it: its
/// two largest classes and the first's share.
fn class_share(r: &Replay, k: usize, c: usize, class: u8) -> f32 {
    let i = k * r.n + c;
    let (a, b, s) = (r.mix_a[i], r.mix_b[i], r.mix_s[i] as f32 / 255.0);
    (if a == class { s } else { 0.0 }) + (if b == class { 1.0 - s } else { 0.0 })
}

// ------------------------------------------------------------------ where a good is made

/// What was made of one good in one year, square by square, as far as the run knows.
pub struct MadeMap {
    /// a figure for every square (nought where none was made, or none is known)
    pub cells: Vec<f32>,
    /// whether the figures are the land's, square by square (a crop, wool,
    /// timber, the pits), or each town's, laid over its own ground (a works' good)
    pub by_land: bool,
    /// whether the land's figures are each working's own, as the record keeps them
    pub from_workings: bool,
    /// what each town made, and what the island made
    pub towns: Vec<f32>,
    pub total: f32,
}

/// Where good `g` was made in year `k`. None where the run keeps no books.
///
/// A good of the works is drawn as each town's figure over the town's own
/// ground. A good of the land is drawn square by square: a pit's or a mine's
/// from the workings, each with what it raised; a crop's, wool's or timber's
/// by sharing each town's figure among the squares of its market by how much
/// of each its class takes and how rich its ground is for it (the record keeps
/// no figure of a farm square by square).
pub fn made_map(r: &Replay, k: usize, g: usize) -> Option<MadeMap> {
    let h = &r.holds;
    let ng = r.header.goods.len();
    if h.made.is_empty() || h.made_towns == 0 || k >= r.filled.min(r.ny) || g >= ng {
        return None;
    }
    let nt = h.made_towns.min(r.header.towns.len());
    let towns: Vec<f32> = (0..nt).map(|t| h.made(k, t, g, ng).unwrap_or(0.0).max(0.0)).collect();
    let total: f32 = towns.iter().sum();
    let mut cells = vec![0.0f32; r.n];
    let sectors: Vec<&str> = h.sectors.iter().filter(|s| s.1 == g).map(|s| s.0.as_str()).collect();
    let mut from_workings = false;
    if !sectors.is_empty() {
        let mut weight = vec![0.0f32; r.n];
        for s in &sectors {
            let Some(class) = replay::sector_class_named(s) else { continue };
            let workings = r.workings.get(k).filter(|w| !w.is_empty() && class >= 5);
            if let Some(ws) = workings {
                // a pit's or a mine's figure is its own
                for w in ws.iter().filter(|w| w.class == class && w.out > 0.0) {
                    if let Some(c) = cells.get_mut(w.cell as usize) {
                        *c += w.out;
                    }
                }
                from_workings = true;
                continue;
            }
            let timber = *s == "forestry" && h.cover.len() == r.ny * r.n;
            let field = if timber { Some(&h.cover[k * r.n..(k + 1) * r.n]) } else { replay::sector_field(s).and_then(|f| h.field(f)) };
            for c in (0..r.n).filter(|&c| land(r, c)) {
                let rich = field.map_or(1.0, |f| f[c] as f32 / 255.0);
                // a wood's figure is its standing timber already; the record's class share would count it twice
                weight[c] += if timber { rich } else { class_share(r, k, c, class) * rich };
            }
        }
        if !from_workings {
            let mut sum = vec![0.0f32; nt];
            for c in 0..r.n {
                if let Some(t) = market(r, k, c).filter(|&t| t < nt) {
                    sum[t] += weight[c];
                }
            }
            for c in 0..r.n {
                if let Some(t) = market(r, k, c).filter(|&t| t < nt && sum[t] > 0.0) {
                    cells[c] = towns[t] * weight[c] / sum[t];
                }
            }
            // a town whose market shows none of the ground its figure came from keeps it at its middle
            for t in (0..nt).filter(|&t| sum[t] <= 0.0 && towns[t] > 0.0) {
                if let Some(c) = r.header.towns.get(t).map(|tw| tw.node).filter(|&c| c < r.n) {
                    cells[c] += towns[t];
                }
            }
        }
        return Some(MadeMap { cells, by_land: true, from_workings, towns, total });
    }
    for c in (0..r.n).filter(|&c| land(r, c)) {
        if let Some(t) = market(r, k, c).filter(|&t| t < nt) {
            if town_ground(r, k, c) || r.header.towns[t].node == c {
                cells[c] = towns[t];
            }
        }
    }
    for t in 0..nt {
        if let Some(c) = Some(r.header.towns[t].node).filter(|&c| c < r.n) {
            cells[c] = cells[c].max(towns[t]);
        }
    }
    Some(MadeMap { cells, by_land: false, from_workings: false, towns, total })
}

// ------------------------------------------------------------------ what the ground holds

/// What the ground holds of one kind in one year, square by square.
pub struct GroundMap {
    /// a figure for every square, on the fields' scale of 0 to 1
    pub cells: Vec<f32>,
    /// the squares whose deposit of this kind has been worked
    pub worked: Vec<bool>,
    /// the share of the island's first deposit of this kind that is left, where known
    pub share_left: Option<f32>,
    /// for timber: whether the figure is the timber standing (a game) or a guess from the woods' share
    pub standing: bool,
    /// GROUND-BRUSH: whether the hand has laid down or taken away any of it by this year
    pub by_hand: bool,
}

/// What the ground holds of kind `i` (`KINDS`) in year `k`. None where the
/// run keeps nothing of it.
pub fn ground_map(r: &Replay, k: usize, i: usize) -> Option<GroundMap> {
    let kind = KINDS.get(i)?;
    let h = &r.holds;
    let mut cells = vec![0.0f32; r.n];
    let mut worked = vec![false; r.n];
    if kind.field == "cover" {
        let standing = h.cover.len() == r.ny * r.n && k < r.filled;
        let pot = h.field("forest_pot");
        if !standing && pot.is_none() {
            return None;
        }
        for c in (0..r.n).filter(|&c| land(r, c)) {
            cells[c] = if standing { h.cover[k * r.n + c] as f32 / 255.0 } else { pot.map_or(0.0, |p| p[c] as f32 / 255.0) * class_share(r, k, c, 3) };
        }
        return Some(GroundMap { cells, worked, share_left: None, standing, by_hand: false });
    }
    let field = h.field(kind.field)?;
    let deposit = replay::DEPOSIT_FIELDS.iter().position(|&f| f == kind.field);
    let left_known = deposit.is_some() && !h.left.is_empty();
    let mut left = vec![1.0f32; r.n];
    if let (Some(d), Some(year)) = (deposit, h.left.get(k)) {
        for &(c, kd, b) in year {
            if kd as usize == d && (c as usize) < r.n {
                left[c as usize] = b as f32 / 255.0;
                worked[c as usize] = true;
            }
        }
    }
    // GROUND-BRUSH: a square whose deposit the hand changed holds what the year says, in place of the field
    let mut laid: Vec<Option<f32>> = Vec::new();
    if let (Some(d), Some(year)) = (deposit, h.now.get(k).filter(|y| !y.is_empty())) {
        laid = vec![None; r.n];
        for &(c, kd, b) in year {
            if kd as usize == d && (c as usize) < r.n {
                laid[c as usize] = Some(b as f32 / 255.0);
            }
        }
    }
    let (mut first, mut now) = (0.0f32, 0.0f32);
    for c in (0..r.n).filter(|&c| land(r, c)) {
        let f = field[c] as f32 / 255.0;
        cells[c] = laid.get(c).copied().flatten().unwrap_or(f) * left[c];
        first += f;
        now += cells[c];
    }
    let share_left = (left_known && first > 0.0).then(|| now / first);
    let by_hand = laid.iter().any(|v| v.is_some());
    Some(GroundMap { cells, worked, share_left, standing: false, by_hand })
}

/// GROUND-BRUSH: what a stroke of the brush would leave on the squares it
/// changes, by the core's own reckoning (`edits::stroke`), on the ground of
/// kind `i` as the viewer holds it in year `k` with the strokes already
/// previewed laid over it: the squares and their new figures, and how rich it
/// lays. `rich` is the richness a stroke under way lays at (None: reckoned
/// from the ground as it stands, as the core reckons it). None where the run
/// keeps nothing of the kind. The viewer's figures are worn and rounded to
/// the fields' 256 steps, so this is a preview: the year's end brings the
/// island's own.
#[allow(clippy::too_many_arguments)]
pub fn preview_stroke(r: &Replay, k: usize, i: usize, preview: &Preview, lay: bool, size: u8, squares: &[usize], rich: Option<f64>) -> Option<(Vec<(usize, f32)>, f64)> {
    let g = ground_map(r, k, i)?;
    let mut old: Vec<f64> = g.cells.iter().map(|&v| v as f64).collect();
    if k >= preview.from {
        for &(_, c, v) in preview.cells.iter().filter(|x| x.0 == i && x.1 < r.n) {
            old[c] = v as f64;
        }
    }
    let lands: Vec<bool> = (0..r.n).map(|c| land(r, c)).collect();
    let (changed, rich) = if KINDS[i].field == "cover" {
        let pot = r.holds.field("forest_pot")?;
        // where woods may stand: the wood's and the heath's share, as the core asks of the wood's and the wild's
        let top = |c: usize| if lands[c] && class_share(r, k, c, 3) + class_share(r, k, c, 4) > 0.02 { pot[c] as f64 / 255.0 } else { 0.0 };
        (isostate_core::edits::stroke(&old, r.w, lay, size, squares, top), 0.0)
    } else {
        let rich = rich.unwrap_or_else(|| isostate_core::edits::richness(&old, &lands));
        (isostate_core::edits::stroke(&old, r.w, lay, size, squares, |c| if lands[c] { rich } else { 0.0 }), rich)
    };
    Some((changed.into_iter().map(|(c, v)| (c, v as f32)).collect(), rich))
}

/// GROUND-BRUSH: the brush's preview painted over the Deposits view's tints,
/// on the view's own scale: a deposit's square as one not worked, a square
/// of timber keeping its market.
fn paint_preview(cells: &mut [u8], p: &Preview, k: usize, which: usize, sc: &Scale) {
    if k < p.from || p.cells.is_empty() {
        return;
    }
    let i = which.min(KINDS.len() - 1);
    let deposit = replay::DEPOSIT_FIELDS.contains(&KINDS[i].field);
    for &(_, c, v) in p.cells.iter().filter(|x| x.0 == i) {
        let Some(px) = cells.get_mut(c * 4..c * 4 + 4) else { continue };
        let col = if v > 0.0 { ramp(&KINDS[i].stops, sc.at(v)) } else { rgb(BLANK) };
        px[..3].copy_from_slice(&col);
        if deposit {
            px[3] = 1;
        }
    }
}

// ------------------------------------------------------------------ one year's figures

/// The figures of year `k` a view's ends are taken over (in the figures' own
/// units): the towns' for prices, wages and a works' good, the squares' for
/// the rest. None where the year shows nothing.
fn sample(r: &Replay, k: usize, kind: Overlay, which: usize) -> Option<Vec<f32>> {
    let lands = (0..r.n).filter(|&c| land(r, c));
    let v: Vec<f32> = match kind {
        Overlay::Areas => return None,
        Overlay::Rent => lands.filter_map(|c| rent_year(r, k, c)).filter(|&v| v > 0.0).collect(),
        Overlay::Reach => lands.filter_map(|c| reach_km(r, k, c)).collect(),
        Overlay::Price => {
            if !made(r, k, which, None) {
                return None;
            }
            markets(r, k).into_iter().filter_map(|t| price(r, k, t, which)).collect()
        }
        Overlay::Wages => markets(r, k).into_iter().filter_map(|t| wage(r, k, t)).collect(),
        Overlay::Made => {
            let m = made_map(r, k, which)?;
            if m.by_land { m.cells.into_iter().filter(|&v| v > 0.0).collect() } else { m.towns.into_iter().filter(|&v| v > 0.0).collect() }
        }
        Overlay::Growth => lands.filter_map(|c| r.joined(k, c)).map(|y| y as f32).collect(),
        Overlay::Residents => lands.filter(|&c| r.joined(k, c).is_some()).map(|c| r.residents(k, c)).filter(|&v| v > 0.0).collect(),
        Overlay::Deposits => ground_map(r, k, which)?.cells.into_iter().filter(|&v| v > 0.0).collect(),
        Overlay::Traffic => {
            if !r.traffic.kept() || k >= r.filled.min(r.ny) {
                return None;
            }
            let t = &r.traffic;
            (0..r.n * 4).map(|l| t.weight(r.n, k, l / 4, l % 4)).filter(|&v| v > 0.0).collect()
        }
    };
    (!v.is_empty()).then_some(v)
}

/// On a scale by ratio, figures under the year's greatest over this set no end.
const TRACE: f32 = 1e4;

/// Year `k`'s own ends for a view, in its scale's measure.
pub fn year_ends(r: &Replay, k: usize, kind: Overlay, which: usize) -> Option<Ends> {
    let spec = kind.spec();
    let mut vals: Vec<f32> = sample(r, k, kind, which)?.into_iter().map(|v| measure(v, spec.log)).collect();
    if spec.log {
        // on a scale by ratio a trace (a town that made a ten-thousandth of what the most
        // did) would stretch the scale over powers of ten nobody can see: it is drawn in
        // the palest tint and sets no end
        let top = vals.iter().copied().fold(f32::MIN, f32::max);
        vals.retain(|&m| m >= top - TRACE.ln());
    }
    ranges::year_ends(vals, spec.pinned.map(|p| measure(p, spec.log)))
}

/// The ends a view is painted with: the range chosen, and the ends it gives,
/// if any year has figures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scaling {
    pub range: Range,
    pub ends: Option<Ends>,
}

impl Scaling {
    /// The ends of a range reckoned from scratch, every year that wants them,
    /// for the tests: the viewer keeps the years (`scaling_for`).
    #[cfg(test)]
    pub fn reckon(r: &Replay, k: usize, kind: Overlay, which: usize, range: Range) -> Scaling {
        Scaling::reckon_trend(r, k, kind, which, range, None)
    }

    /// The same, of a view's trend over `span` where one is given.
    #[cfg(test)]
    pub fn reckon_trend(r: &Replay, k: usize, kind: Overlay, which: usize, range: Range, span: Option<Span>) -> Scaling {
        let upto = match range {
            Range::Year => k + 1,
            Range::Thirty => k + 1,
            Range::Whole => r.filled.min(r.ny),
        };
        let years: Vec<Option<Ends>> = (0..upto).map(|j| if range == Range::Year && j != k { None } else { ends_of(r, j, kind, which, span) }).collect();
        Scaling { range, ends: ranges::ends(range, &years, k) }
    }
}

// ------------------------------------------------------------------ one year's overlay

/// What the legend card says.
#[derive(Clone, Debug, PartialEq)]
pub struct Legend {
    pub title: String,
    pub note: String,
    /// the tints from low to high; empty where there is no scale
    pub ramp: Vec<[u8; 3]>,
    /// round figures along the scale: where each stands from 0 to 1, and its words
    pub ticks: Vec<(f32, String)>,
    /// what the two ends of the bar stand for: "and below" and "and above",
    /// or the view's own word where its nature sets an end ("at the market")
    pub low: String,
    pub high: String,
    /// the two ends in figures, for a script's `map` line
    pub ends: String,
    /// the range the colours are spread over, where the view has one to choose
    pub range: Option<Range>,
    /// tints that stand for a word and not a figure, named beside a swatch
    /// under the bar: under a trend, "new since 1890" and "gone since 1890"
    pub keys: Vec<([u8; 3], String)>,
    /// under a trend, the towns rising fastest and falling fastest
    pub movers: Option<(Vec<trend::Mover>, Vec<trend::Mover>)>,
}

pub struct Wash {
    /// a tint for every cell and, in alpha, its region (0: leave the ground alone)
    pub cells: Vec<u8>,
    pub legend: Legend,
    /// how far the tints blend across a square's edge
    pub soft: f32,
    /// ink lines where regions meet
    pub lines: bool,
}

fn ramp_bar(stops: &[u32; 5]) -> Vec<[u8; 3]> {
    (0..24).map(|i| ramp(stops, i as f32 / 23.0)).collect()
}

/// The legend's figures for a scale: its round figures, where each stands,
/// and the two ends in words.
fn scale_words(kind: Overlay, sc: &Scale) -> (Vec<(f32, String)>, String) {
    let values = ranges::ticks(sc.lo, sc.hi, sc.log);
    // money is written alike along one scale: £2 among £1.80 and £2.20 is £2.00
    let whole = values.iter().all(|v| v.fract().abs() < 1e-3);
    let words = |v: f32| if matches!(kind, Overlay::Price | Overlay::Wages) && !whole { coin(v) } else { tick_words(kind, v) };
    let ticks = values.iter().map(|&v| (sc.at(v), words(v))).collect();
    (ticks, format!("{} to {}", tick_words(kind, sc.lo), tick_words(kind, sc.hi)))
}

/// The overlay for year index `k`: tints, and the legend that explains them.
/// With `month`, prices and wages are that month's and not the year's.
/// `which` is the good (prices, made) or the kind of ground (deposits), and
/// `scaling` the range and its ends, for the views that colour by a number.
pub fn wash(r: &Replay, k: usize, kind: Overlay, which: usize, month: Option<&Month>, scaling: Scaling) -> Wash {
    let year = r.y0 + k as i32;
    let mut cells = vec![0u8; r.n * 4];
    // every land cell carries its market as its region, so the shader can rule the boundaries
    let region = |c: usize| market(r, k, c).map_or(255, |t| (t + 1).min(254) as u8);
    let mut put = |c: usize, col: [u8; 3]| cells[c * 4..c * 4 + 4].copy_from_slice(&[col[0], col[1], col[2], region(c)]);
    let lands = (0..r.n).filter(|&c| land(r, c));
    let none = |title: String, note: String| Legend { title, note, ramp: Vec::new(), ticks: Vec::new(), low: String::new(), high: String::new(), ends: String::new(), range: None, keys: Vec::new(), movers: None };
    let spec = kind.spec();
    // the scale the range gives; a view with no figures at all says so before it is wanted
    let scale = scaling.ends.map(|e| Scale::of(e, spec.log));
    let words = |sc: &Scale, stops: &[u32; 5], pinned_word: Option<&str>| -> (Vec<[u8; 3]>, Vec<(f32, String)>, String, String, String) {
        let (ticks, ends) = scale_words(kind, sc);
        (ramp_bar(stops), ticks, pinned_word.unwrap_or("and below").to_string(), "and above".to_string(), ends)
    };
    let legend = |title: String, note: String, stops: &[u32; 5], sc: &Scale, pinned_word: Option<&str>| {
        let (ramp, ticks, low, high, ends) = words(sc, stops, pinned_word);
        Legend { title, note, ramp, ticks, low, high, ends, range: Some(scaling.range), keys: Vec::new(), movers: None }
    };
    match kind {
        Overlay::Rent => {
            let title = format!("Land rent in {year}");
            let Some(sc) = scale else {
                return Wash { cells, legend: none(title, "This record does not hold the rent of the land.".into()), soft: 0.5, lines: false };
            };
            for c in lands {
                let v = rent_year(r, k, c).unwrap_or(0.0);
                put(c, ramp(&SEPIA, sc.at(v)));
            }
            let note = "What the ground of each square earns its owners in a year, from farms, mines, works and town houses.".into();
            Wash { cells, legend: legend(title, note, &SEPIA, &sc, None), soft: 0.5, lines: false }
        }
        Overlay::Reach => {
            let title = format!("The way to market in {year}");
            let (Some(sc), false) = (scale, r.dist.is_empty()) else {
                return Wash { cells, legend: none(title, "This record does not hold the way to market.".into()), soft: 0.5, lines: false };
            };
            for c in lands {
                // land with no way to a market is as far as can be
                put(c, ramp(&SLATE, reach_km(r, k, c).map_or(1.0, |v| sc.at(v))));
            }
            let note = "How far each square is from the market it trades in, counted as kilometres of flat open country. Roads, rivers, the sea and railways shorten the way; hills and woods lengthen it.".into();
            Wash { cells, legend: legend(title, note, &SLATE, &sc, Some("at the market")), soft: 0.5, lines: false }
        }
        Overlay::Areas => {
            let tint = area_tints(r, k);
            for c in lands {
                put(c, rgb(market(r, k, c).map_or(BLANK, |t| AREAS[tint[t]])));
            }
            let n = markets(r, k).len();
            let legend = none(
                format!("Market areas in {year}"),
                format!("Each tint is the country that buys and sells in one town's market, and the lines are where one town's pull gives way to the next. {n} market{}.", if n == 1 { "" } else { "s" }),
            );
            Wash { cells, legend, soft: 0.06, lines: true }
        }
        Overlay::Price | Overlay::Wages => {
            let good = which;
            let is_price = kind == Overlay::Price;
            let label = r.header.goods.get(good).map_or("goods".to_string(), |g| g.label.to_lowercase());
            let value = |t: usize| match month {
                Some(m) => if is_price { m.months.price(m.t, t, good) } else { m.months.wage(m.t, t) }.filter(|v| v.is_finite() && *v > 0.0),
                None => if is_price { price(r, k, t, good) } else { wage(r, k, t) },
            };
            let when = month.map_or(format!("{year}"), |m| m.label.clone());
            let title = if is_price { format!("The price of {label} in {when}") } else { format!("Wages in {when}") };
            // a good no works has made yet has prices in the record that nothing was sold at: no map of those
            let unmade = is_price && !made(r, k, good, month);
            // the cheapest and the dearest market, named in the note
            let towns: Vec<(usize, f32)> = if unmade { Vec::new() } else { markets(r, k).into_iter().filter_map(|t| value(t).map(|v| (t, v))).collect() };
            let lo = towns.iter().copied().min_by(|a, b| a.1.total_cmp(&b.1));
            let hi = towns.iter().copied().max_by(|a, b| a.1.total_cmp(&b.1));
            let (Some(lo), Some(hi), Some(sc)) = (lo, hi, scale) else {
                for c in lands {
                    put(c, rgb(BLANK));
                }
                let note = if !is_price {
                    "This record does not hold the towns' wages.".to_string()
                } else if unmade {
                    format!("No {label} has been made on the island by {when}, so no market has a price for it.")
                } else if r.town_price.is_empty() {
                    "This record does not hold the towns' prices.".to_string()
                } else {
                    format!("No market has a price for {label} in {when}.")
                };
                return Wash { cells, legend: none(title, note), soft: 0.06, lines: true };
            };
            let stops = if is_price { &MADDER } else { &MULBERRY };
            for c in lands {
                put(c, market(r, k, c).and_then(value).map_or(rgb(BLANK), |v| ramp(stops, sc.at(v))));
            }
            let lead = match (is_price, month.is_some()) {
                (true, false) => "Each square takes the price in the market it trades in, the year's average. Carriage to and from the market is not counted.",
                (true, true) => "Each square takes the price in the market it trades in, as it stood at the month's end. Carriage to and from the market is not counted.",
                (false, false) => "What a month's work pays in each town, the year's average, shown across the town's market area.",
                (false, true) => "What the month's work paid in each town, shown across the town's market area.",
            };
            let note = if lo.0 == hi.0 {
                format!("{lead} {} at {}.", coin(lo.1), town_name(r, lo.0))
            } else if is_price {
                format!("{lead} The cheapest market is {} at {}, the dearest {} at {}.", town_name(r, lo.0), coin(lo.1), town_name(r, hi.0), coin(hi.1))
            } else {
                format!("{lead} The lowest pay is at {}, {}, the highest at {}, {}.", town_name(r, lo.0), coin(lo.1), town_name(r, hi.0), coin(hi.1))
            };
            Wash { cells, legend: legend(title, note, stops, &sc, None), soft: 0.06, lines: true }
        }
        Overlay::Made => {
            let good = which;
            let label = r.header.goods.get(good).map_or("goods".to_string(), |g| g.label.to_lowercase());
            // (services, chemicals and electricals are many things)
            let title = format!("Where {label} {} made in {year}", if label.ends_with('s') { "were" } else { "was" });
            let blank = |cells: &mut Vec<u8>| {
                for c in (0..r.n).filter(|&c| land(r, c)) {
                    let reg = market(r, k, c).map_or(255, |t| (t + 1).min(254) as u8);
                    cells[c * 4..c * 4 + 4].copy_from_slice(&[0xd9, 0xd3, 0xc4, reg]);
                }
            };
            let Some(m) = made_map(r, k, good) else {
                blank(&mut cells);
                return Wash { cells, legend: none(title, "This record does not keep what each town makes: the books of account, which hold it, are kept under the game's rules.".into()), soft: 0.06, lines: true };
            };
            let (Some(sc), true) = (scale, m.total > 0.0) else {
                blank(&mut cells);
                return Wash { cells, legend: none(title, format!("Nobody made any {label} on the island in {year}.")), soft: 0.06, lines: true };
            };
            for c in lands {
                let v = m.cells[c];
                put(c, if v > 0.0 { ramp(&OCHRE, sc.at(v)) } else { rgb(BLANK) });
            }
            let top = (0..m.towns.len()).max_by(|&a, &b| m.towns[a].total_cmp(&m.towns[b]));
            let most = top.map_or(String::new(), |t| format!(" Most was made at {}, {}.", town_name(r, t), amount(m.towns[t])));
            let making = m.towns.iter().filter(|&&v| v > 0.0).count();
            let note = if m.from_workings {
                format!("What each working raised in the year, at its pit head, in the good's own units. {} made on the island, by {making} town{}.{most}", amount(m.total), if making == 1 { "" } else { "s" })
            } else if m.by_land {
                format!("What each square grew or felled in the year, in the good's own units: each town's figure shared among the squares of its market by how much of each is given to it and how rich its ground is. {} made on the island.{most}", amount(m.total))
            } else {
                format!("What each town's works made in the year, in the good's own units, laid over the town's own ground. {} made on the island, in {making} town{}.{most}", amount(m.total), if making == 1 { "" } else { "s" })
            };
            Wash { cells, legend: legend(title, note, &OCHRE, &sc, None), soft: 0.06, lines: true }
        }
        // (task A) the two views of town ground that is kept (docs/rules/towns.md)
        Overlay::Growth | Overlay::Residents => {
            let growth = kind == Overlay::Growth;
            let title = if growth { format!("Town ground in {year}, by when it joined") } else { format!("Residents of town ground in {year}") };
            if !r.kept() {
                // (the land is left blank, each square with its market, as on any other view)
                for c in lands {
                    put(c, rgb(BLANK));
                }
                return Wash { cells, legend: none(title, "This record does not hold the towns' ground: under its rules a town does not keep the ground it builds on.".into()), soft: 0.06, lines: false };
            }
            // the whole run of when ground joined is its own scale, the same in every year, so a
            // square keeps its tint for as long as its figure stands: from the oldest ground the
            // record dates to the run's last year. Residents go by ratio, as rent does: for most of
            // the century the towns' ground holds 400 to 700, and on an even scale it was all one pale tint
            let last = r.y0 + r.ny as i32 - 1;
            let sc = if growth && scaling.range == Range::Whole {
                Scale { lo: OLDEST_GROUND, hi: last as f32, log: false }
            } else if let Some(sc) = scale {
                sc
            } else {
                for c in lands {
                    put(c, rgb(BLANK));
                }
                return Wash { cells, legend: none(title, "No square is town ground this year.".into()), soft: 0.06, lines: false };
            };
            let stops = if growth { &BRICK } else { &INDIGO };
            let (mut squares, mut people) = (0usize, 0.0f32);
            for c in lands {
                let tint = r.joined(k, c).map(|y| {
                    squares += 1;
                    people += r.residents(k, c);
                    ramp(stops, sc.at(if growth { y as f32 } else { r.residents(k, c) }))
                });
                put(c, tint.unwrap_or(rgb(BLANK)));
            }
            let s = if squares == 1 { "" } else { "s" };
            let note = if growth {
                format!("When each square of town ground joined its town. A town keeps the ground it builds on and grows outward from it, so its oldest ground is its middle. {squares} square{s} of town ground.")
            } else {
                format!("The people living on each square of town ground. Ground holds 300 people the year it joins its town and fills as it ages. {} people on {squares} square{s}.", ui::thousands(people))
            };
            let pinned = (growth && scaling.range == Range::Whole).then_some("and before");
            let mut l = legend(title, note, stops, &sc, pinned);
            if growth && scaling.range == Range::Whole {
                l.high = format!("{last}");
            }
            Wash { cells, legend: l, soft: 0.06, lines: false }
        }
        Overlay::Deposits => {
            let i = which.min(KINDS.len() - 1);
            let kd = &KINDS[i];
            let title = match kd.word {
                "timber" => format!("Standing timber in {year}"),
                "soil" => format!("The soil in {year}"),
                "grazing" => format!("The grazing in {year}"),
                _ => format!("{} in the ground in {year}", kd.label),
            };
            let Some(g) = ground_map(r, k, i) else {
                for c in lands {
                    put(c, rgb(BLANK));
                }
                return Wash { cells, legend: none(title, "This record does not hold what the ground holds.".into()), soft: 0.5, lines: false };
            };
            let Some(sc) = scale else {
                for c in lands {
                    put(c, rgb(BLANK));
                }
                return Wash { cells, legend: none(title, format!("There is no {} anywhere on the island.", kd.label.to_lowercase())), soft: 0.5, lines: false };
            };
            let is_deposit = replay::DEPOSIT_FIELDS.contains(&kd.field);
            for c in lands {
                let v = g.cells[c];
                let col = if v > 0.0 { ramp(&kd.stops, sc.at(v)) } else { rgb(BLANK) };
                // the squares that have been worked are a region of their own, ruled round in ink
                let reg = if is_deposit { if g.worked[c] { 2 } else { 1 } } else { market(r, k, c).map_or(255, |t| (t + 1).min(254) as u8) };
                cells[c * 4..c * 4 + 4].copy_from_slice(&[col[0], col[1], col[2], reg]);
            }
            let lname = kd.label.to_lowercase();
            let note = match (kd.word, is_deposit) {
                ("timber", _) if g.standing => "The timber standing on each square as the year ends, on the scale the ground's richness is kept in: 1 is as thick a wood as the richest ground could carry.".to_string(),
                ("timber", _) => "How much timber each square could carry, as much of it as is wood: this record does not keep the timber standing.".to_string(),
                ("soil", _) => "How fertile each square's soil is, on the scale the ground's richness is kept in, 1 the richest there could be.".to_string(),
                ("grazing", _) => "How good each square's grazing is, on the scale the ground's richness is kept in, 1 the best there could be.".to_string(),
                (_, true) => match g.share_left {
                    // GROUND-BRUSH: with the hand's strokes it is no longer only what is left
                    Some(s) if g.by_hand => format!("How rich the {lname} under each square is now: the richness the hands would meet there, which falls as it is worked, with what the hand has laid down or taken away. The squares ruled round in ink have been worked. With the hand's strokes, the ground holds {:.0}% of the {lname} the island began with.", s * 100.0),
                    Some(s) => format!("How rich the {lname} under each square is now: the richness the hands would meet there, which falls as it is worked. The squares ruled round in ink have been worked. Of the {lname} the island began with, {:.0}% is left.", s * 100.0),
                    None => format!("How rich the {lname} under each square was when the island was made: this record does not keep what has been worked out of it."),
                },
                _ => String::new(),
            };
            Wash { cells, legend: legend(title, note, &kd.stops, &sc, Some("none")), soft: 0.5, lines: is_deposit }
        }
        Overlay::Traffic => {
            // the land as plain paper, so that the ribbons laid over it (`traffic.rs`) read
            let title = format!("What the ways carried in {year}");
            for c in lands {
                put(c, rgb(TRAFFIC_PAPER));
            }
            if !r.traffic.kept() {
                return Wash { cells, legend: none(title, "This record does not keep what the ways carry: the traffic is kept under the game's rules from 25 November 2026.".into()), soft: 0.5, lines: false };
            }
            let Some(sc) = scale else {
                return Wash { cells, legend: none(title, format!("Nothing was carried over the ways in {year}.")), soft: 0.5, lines: false };
            };
            let t = &r.traffic;
            // what each way carried, weight times length, and the busiest link
            let (mut by, mut top) = ([0.0f64; 4], (0.0f32, 0usize, 0usize));
            for c in 0..r.n {
                for s in 0..4 {
                    let w = t.weight(r.n, k, c, s);
                    if w > 0.0 {
                        by[t.mode(r.n, k, c, s) as usize & 3] += w as f64 * isostate_core::traffic::slot_len(s);
                        if w > top.0 {
                            top = (w, c, s);
                        }
                    }
                }
            }
            let all: f64 = by.iter().sum();
            // the lorry (the core's docs/rules/lorry.md): the record's two bits of mode say
            // only "by road", so the year's share of the carting's hauling done by lorry,
            // which a run with the lorry keeps as a series, says how much of that was lorries
            let lorries = crate::replay::annual(r, "lorry_share", k).filter(|v| v.is_finite() && *v >= 0.005);
            let words = if lorries.is_some() { ["by road", "by boat", "by rail", "by sea"] } else { ["by cart", "by boat", "by rail", "by sea"] };
            let mut shares: Vec<(f64, &str)> = by.iter().zip(words).map(|(&v, w)| (v / all.max(1e-9), w)).filter(|s| s.0 >= 0.005).collect();
            shares.sort_by(|a, b| b.0.total_cmp(&a.0));
            let shares: Vec<String> = shares.iter().map(|(v, w)| format!("{:.0}% {w}", v * 100.0)).collect();
            let busiest = match market(r, k, top.1) {
                Some(town) => format!(" The busiest way, near {}, carried {} {}.", town_name(r, town), amount(top.0), words[t.mode(r.n, k, top.1, top.2) as usize & 3]),
                None => format!(" The busiest way carried {} {}.", amount(top.0), words[t.mode(r.n, k, top.1, top.2) as usize & 3]),
            };
            let (brown, by_lorry) = match lorries {
                Some(v) => ("brown by road, cart or lorry", format!(" By road, lorries did {:.0}% of the carting's hauling this year, and carts and horses the rest.", v * 100.0)),
                None => ("brown by cart", String::new()),
            };
            let note = format!(
                "Each way between two squares is drawn as wide as the weight carried over it in the year, both ways together, in the goods' units: {brown}, blue by river or canal, black by rail, pale blue by sea. Of the weight carried, times how far: {}.{by_lorry}{busiest}",
                shares.join(", ")
            );
            Wash { cells, legend: legend(title, note, &TRAFFIC_INK, &sc, None), soft: 0.5, lines: false }
        }
    }
}

/// The paper the Traffic view lays its ribbons on, and its scale's ink (the ribbons' own by cart).
const TRAFFIC_PAPER: u32 = 0xe9e1d0;
const TRAFFIC_INK: [u32; 5] = [0xe6dccb, 0xd1b496, 0xb58763, 0x9e6640, 0x8a4f2a];

// ------------------------------------------------------------------ the trend, painted

/// A trend's tints: pale where nothing moves, Prussian blue as it rises and
/// brick red as it falls, the two sides alike in how fast they darken. Ground
/// new in the span is saffron and ground gone is charcoal, neither on the scale.
const CALM: u32 = 0xf1ede3;
const RISE: [u32; 5] = [CALM, 0xc8d9df, 0x92b5c6, 0x5a87a6, 0x2f5a80];
const FALL: [u32; 5] = [CALM, 0xeacdbb, 0xd89c80, 0xb8664c, 0x84392c];
pub const NEW: u32 = 0xe2b43c;
pub const GONE: u32 = 0x4f4a47;

/// The tint at `t` from 0 (falling fastest) through 0.5 (still) to 1 (rising fastest).
fn diverge(t: f32) -> [u8; 3] {
    if t >= 0.5 { ramp(&RISE, (t - 0.5) * 2.0) } else { ramp(&FALL, (0.5 - t) * 2.0) }
}

/// A square's tint under a trend on scale `sc`, BLANK where nothing was there at either end.
pub fn change_tint(c: trend::Change, sc: &Scale) -> [u8; 3] {
    match c {
        trend::Change::Nothing => rgb(BLANK),
        trend::Change::New => rgb(NEW),
        trend::Change::Gone => rgb(GONE),
        trend::Change::Rate(v) => diverge(sc.at(v)),
    }
}

/// A view coloured by its change over `span` to year `k`, from the
/// reckoning in `trend.rs`, with the movers for the card. `scaling` holds the
/// ends in per cent a year, even about nought (`trend::trend_ends`), taken
/// under the view's range as its plain colours are.
pub fn trend_wash(r: &Replay, k: usize, kind: Overlay, which: usize, span: Span, scaling: Scaling) -> Wash {
    let year = r.y0 + k as i32;
    let n = span.years();
    let mut cells = vec![0u8; r.n * 4];
    let region = |c: usize| market(r, k, c).map_or(255, |t| (t + 1).min(254) as u8);
    let label = r.header.goods.get(which).map_or("goods".to_string(), |g| g.label.to_lowercase());
    let what = match kind {
        Overlay::Residents => "Residents".to_string(),
        Overlay::Wages => "Wages".to_string(),
        Overlay::Rent => "Land rent".to_string(),
        Overlay::Price => format!("The price of {label}"),
        Overlay::Made => format!("The making of {label}"),
        _ => kind.chip().to_string(),
    };
    let (soft, lines) = match kind {
        Overlay::Rent => (0.5, false),
        Overlay::Residents => (0.06, false),
        _ => (0.06, true),
    };
    let none = |title: String, note: String| Legend { title, note, ramp: Vec::new(), ticks: Vec::new(), low: String::new(), high: String::new(), ends: String::new(), range: None, keys: Vec::new(), movers: None };
    let blank = |cells: &mut Vec<u8>| {
        for c in (0..r.n).filter(|&c| land(r, c)) {
            let b = rgb(BLANK);
            cells[c * 4..c * 4 + 4].copy_from_slice(&[b[0], b[1], b[2], region(c)]);
        }
    };
    let Some(from) = k.checked_sub(n) else {
        blank(&mut cells);
        let note = format!("A change over {n} years can be shown from {}: the run begins in {}.", r.y0 + n as i32, r.y0);
        return Wash { cells, legend: none(format!("{what}: change over {n} years"), note), soft, lines };
    };
    let y0 = r.y0 + from as i32;
    let title = format!("{what}: change a year, {y0} to {year}");
    let Some(m) = trend::trend_map(r, k, kind, which, span) else {
        // nothing this year (no record of it, a good not made): the plain view's words say why
        blank(&mut cells);
        let plain = wash(r, k, kind, which, None, Scaling { range: scaling.range, ends: None });
        return Wash { cells, legend: none(title, plain.legend.note), soft, lines };
    };
    let e = scaling.ends.unwrap_or(Ends { lo: -trend::LEAST_END, hi: trend::LEAST_END });
    let sc = Scale { lo: e.lo, hi: e.hi, log: false };
    for c in (0..r.n).filter(|&c| land(r, c)) {
        let col = change_tint(m.cells[c], &sc);
        cells[c * 4..c * 4 + 4].copy_from_slice(&[col[0], col[1], col[2], region(c)]);
    }
    let ticks = ranges::ticks(sc.lo, sc.hi, false).into_iter().map(|v| (sc.at(v), trend::rate_words(v))).collect();
    let (new, gone) = m.new_and_gone();
    let unit = if m.by_town { "town" } else { "square" };
    let s = |n: usize| if n == 1 { "" } else { "s" };
    let lead = match kind {
        Overlay::Residents => format!("The people living on each square of town ground in {year} against {y0}"),
        Overlay::Wages => format!("What a month's work pays in each town in {year} against {y0}, across its market area"),
        Overlay::Rent => format!("What the ground of each square earns in a year, {year} against {y0}"),
        Overlay::Price => format!("The price of {label} in each town's market in {year} against {y0}, the years' averages"),
        Overlay::Made if m.by_town => format!("What each town's works made of {label} in {year} against {y0}, over the town's own ground"),
        _ => format!("How much {label} each square grew or raised in {year} against {y0}"),
    };
    let note = format!("{lead}, as a rate in per cent a year: blue where it rose, red where it fell, pale where it held. {new} {unit}{} new since {y0}, {gone} gone.", s(new));
    let keys = vec![(rgb(NEW), format!("new since {y0}")), (rgb(GONE), format!("gone since {y0}"))];
    let legend = Legend {
        title,
        note,
        ramp: (0..24).map(|i| diverge(i as f32 / 23.0)).collect(),
        ticks,
        low: "and falling faster".to_string(),
        high: "and rising faster".to_string(),
        ends: format!("{} to {} a year", trend::rate_words(sc.lo), trend::rate_words(sc.hi)),
        range: Some(scaling.range),
        keys,
        movers: Some(trend::movers(r, &m)),
    };
    Wash { cells, legend, soft, lines }
}

/// What the overlay says of square `c` in year index `k`, as a sentence for the
/// inspector. None where it has nothing to say (the sea, or no record).
pub fn said(r: &Replay, k: usize, c: usize, kind: Overlay, which: usize, month: Option<&Month>) -> Option<String> {
    if c >= r.n || !land(r, c) {
        return None;
    }
    let town = market(r, k, c);
    let good = which;
    Some(match kind {
        Overlay::Rent => match rent_year(r, k, c)? {
            // the record's scale stops at its top step, which the middle of a town reaches
            v if r.rent[k * r.n + c] == u8::MAX => format!("Rent: the ground of this square earns {} a year or more.", about(v)),
            v if v > 0.0 => format!("Rent: the ground of this square earns about {} a year.", about(v)),
            _ => "Rent: the ground of this square earns nothing.".to_string(),
        },
        Overlay::Areas => match town {
            Some(t) => {
                let n = (0..r.n).filter(|&d| land(r, d) && market(r, k, d) == Some(t)).count();
                format!("Market: it buys and sells at {}, one of {} squares that do.", town_name(r, t), n)
            }
            None => "Market: it trades in no town's market.".to_string(),
        },
        Overlay::Reach => match (reach_km(r, k, c), town) {
            (Some(v), Some(t)) if v < 0.25 => format!("Way to market: {}'s market is here.", town_name(r, t)),
            (Some(v), Some(t)) => format!("Way to market: {} is as far as {} km of flat open country.", town_name(r, t), if v < 10.0 { format!("{v:.1}") } else { format!("{v:.0}") }),
            _ if r.dist.is_empty() => return None,
            _ => "Way to market: there is no way from here to any market.".to_string(),
        },
        Overlay::Price => {
            let g = r.header.goods.get(good)?;
            // the record holds a price, not whether any was sold at it, and a good may be
            // one thing or many (coal, rails), so the sentence commits to neither
            let of = |t: usize| match month {
                Some(m) => m.months.price(m.t, t, good).filter(|p| p.is_finite() && *p > 0.0),
                None => price(r, k, t, good),
            };
            let then = month.map_or(String::new(), |m| format!(" in {}", m.label));
            if !made(r, k, good, month) {
                return Some(format!("Price: no {} has been made on the island yet.", g.label.to_lowercase()));
            }
            match town.map(|t| (t, of(t))) {
                Some((t, Some(p))) => format!("Price: {} for {} at {}{then}, the market this square trades in.", coin(p), g.label.to_lowercase(), town_name(r, t)),
                Some((t, None)) => format!("Price: {}'s market has no price for {} {}.", town_name(r, t), g.label.to_lowercase(), if month.is_some() { "this month" } else { "this year" }),
                None => "Price: it trades in no town's market.".to_string(),
            }
        }
        Overlay::Made => {
            let g = r.header.goods.get(good)?;
            let label = g.label.to_lowercase();
            let m = made_map(r, k, good)?;
            match (m.by_land, town) {
                (true, _) if m.cells[c] > 0.0 => format!("Made here: about {} of {label} {} this square in the year.", amount(m.cells[c]), if m.from_workings { "was raised at the pit head on" } else { "came off" }),
                (true, Some(t)) => format!("Made here: none of {label} on this square; {} made {} in all.", town_name(r, t), amount(m.towns.get(t).copied().unwrap_or(0.0))),
                (false, Some(t)) => format!("Made here: {} made {} of {label} in the year.", town_name(r, t), amount(m.towns.get(t).copied().unwrap_or(0.0))),
                (_, None) => format!("Made here: none of {label}, and it trades in no town's market."),
            }
        }
        Overlay::Wages => {
            let of = |t: usize| match month {
                Some(m) => m.months.wage(m.t, t).filter(|w| w.is_finite() && *w > 0.0),
                None => wage(r, k, t),
            };
            match town.and_then(|t| of(t).map(|w| (t, w))) {
                Some((t, w)) => match month {
                    Some(m) => format!("Wages: a month's work paid {} at {} in {}.", coin(w), town_name(r, t), m.label),
                    None => format!("Wages: a month's work pays {} at {}.", coin(w), town_name(r, t)),
                },
                None => return None,
            }
        }
        // (task A)
        Overlay::Growth | Overlay::Residents if !r.kept() => return None,
        Overlay::Growth => match (r.joined(k, c), town) {
            (Some(y), Some(t)) if y < r.y0 => format!("Town ground: part of {} since about {y}.", town_name(r, t)),
            (Some(y), Some(t)) => format!("Town ground: it joined {} in {y}.", town_name(r, t)),
            (Some(y), None) => format!("Town ground: since {y}."),
            (None, _) => "Town ground: this square is not town ground.".to_string(),
        },
        Overlay::Residents => match r.joined(k, c) {
            Some(_) => format!("Residents: about {} people live on this square.", ui::thousands(r.residents(k, c))),
            None => "Residents: this square is not town ground.".to_string(),
        },
        Overlay::Deposits => {
            let i = which.min(KINDS.len() - 1);
            let g = ground_map(r, k, i)?;
            let first = r.holds.field(KINDS[i].field).map(|f| f[c] as f32 / 255.0);
            let name = KINDS[i].label;
            match KINDS[i].word {
                "timber" if g.standing => format!("{name}: {:.2} on the scale of the richest ground.", g.cells[c]),
                "timber" => format!("{name}: it could carry {:.2}; this record does not keep what stands.", g.cells[c]),
                _ if g.cells[c] <= 0.0 && first.is_none_or(|f| f <= 0.0) => format!("{name}: none."),
                _ if g.worked[c] => format!("{name}: {:.2} now, of {:.2} when the island was made; it has been worked.", g.cells[c], first.unwrap_or(0.0)),
                _ => format!("{name}: {:.2}, on the scale of the richest ground.", g.cells[c]),
            }
        }
        Overlay::Traffic => {
            let t = &r.traffic;
            if !t.kept() {
                return None;
            }
            // the square's eight ways: its own four links and its neighbours' four that end here
            let dirs = ["east", "south", "south-east", "south-west"];
            let back = ["west", "north", "north-west", "north-east"];
            let words = ["by cart", "by boat", "by rail", "by sea"];
            let mut ways: Vec<(f32, &str, u8)> = Vec::new();
            for s in 0..4 {
                ways.push((t.weight(r.n, k, c, s), dirs[s], t.mode(r.n, k, c, s)));
                let (dx, dy) = isostate_core::traffic::SLOTS[s];
                let (x, y) = ((c % r.w) as isize - dx, (c / r.w) as isize - dy);
                if x >= 0 && y >= 0 && (x as usize) < r.w && (y as usize) < r.h {
                    let o = y as usize * r.w + x as usize;
                    ways.push((t.weight(r.n, k, o, s), back[s], t.mode(r.n, k, o, s)));
                }
            }
            ways.retain(|w| w.0 > 0.0);
            ways.sort_by(|a, b| b.0.total_cmp(&a.0));
            match ways.first() {
                None => "Traffic: nothing was carried to or from this square.".to_string(),
                Some(&(w, d, m)) => {
                    let more = ways.get(1).map_or(String::new(), |&(w2, d2, m2)| format!("; the next, {}, {} {}", d2, amount(w2), words[m2 as usize & 3]));
                    format!("Traffic: the busiest way from this square, to the {d}, carried {} {} in the year{more}.", amount(w), words[m as usize & 3])
                }
            }
        }
    })
}

// ------------------------------------------------------------------ the legend card

#[derive(Component)]
struct LegendCard;

/// Room for a good's name between Back and Next on the price map's card: a label of 24 characters.
const GOOD_BOX_W: f32 = 172.0;
/// The card's width inside its padding.
const CARD_W: f32 = 396.0;

/// Choose an overlay (or none).
#[derive(Component, Clone, Copy)]
pub struct OverlayChip(pub Option<Overlay>);

/// The good before or after the one whose price is mapped (or the kind of ground).
#[derive(Component, Clone, Copy)]
pub struct GoodStep(pub i32);

/// Choose how the view's colours are spread.
#[derive(Component, Clone, Copy)]
pub struct RangeChip(pub Range);

/// The Trend switch: off, or the years it looks back over.
#[derive(Component, Clone, Copy)]
pub struct TrendChip(pub Option<Span>);

/// A town among the movers: a click takes the camera there.
#[derive(Component, Clone, Copy)]
pub struct MoverRow(pub usize);

pub fn plugin(app: &mut App) {
    app.add_systems(PostStartup, setup).add_systems(Update, (input, paint, legend_card).chain());
}

fn setup(mut commands: Commands) {
    // above the timeline, beside the side panel; its contents are made by legend_card
    commands.spawn((
        ui::card(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(408.0),
            bottom: Val::Px(70.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            padding: UiRect::axes(Val::Px(14.0), Val::Px(9.0)),
            border_radius: BorderRadius::all(Val::Px(14.0)),
            ..default()
        }),
        LegendCard,
    ));
}

/// M steps through the overlays (with Shift, backwards) and G through the
/// goods of the price and made maps, or the kinds of ground; the chips on the
/// card do the same by hand, and choose the range.
#[allow(clippy::too_many_arguments)]
fn input(
    keys: Res<ButtonInput<KeyCode>>,
    data: Res<Data>,
    mut ov: ResMut<MapOverlay>,
    chips: Query<(&Interaction, &OverlayChip), Changed<Interaction>>,
    steps: Query<(&Interaction, &GoodStep), Changed<Interaction>>,
    ranges: Query<(&Interaction, &RangeChip), Changed<Interaction>>,
    trends: Query<(&Interaction, &TrendChip), Changed<Interaction>>,
    mut movers: Query<(&Interaction, &MoverRow, &mut BackgroundColor), Changed<Interaction>>,
    cards: Res<crate::game::CardOpen>,
    tl: Res<Timeline>,
    mut o: ResMut<crate::Orbit>,
    mut picked: ResMut<crate::Picked>,
) {
    for (i, t) in &trends {
        if *i == Interaction::Pressed && ov.trend != t.0 {
            ov.trend = t.0;
        }
    }
    // a town among the movers takes the camera there and opens its note, as a town's row in the
    // panels does; from a little higher than they take it (22 squares, not 9), so that the town's
    // ground and its market's tints read round the houses
    for (i, row, mut bg) in &mut movers {
        bg.0 = if *i == Interaction::None { Color::NONE } else { ui::PAPER_2 };
        if *i == Interaction::Pressed {
            if let Some(&(x, z)) = data.r.header.towns.get(row.0).and_then(|t| data.cen.get(t.node)) {
                o.target = Vec3::new(x, crate::world::height_at(&data.g, x, z).max(0.0), z);
                o.dist = o.dist.min(22.0);
                picked.what = Some(crate::Pick::Town(row.0));
            }
        }
    }
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    // while a card wants an answer the keys are the card's, as with the panels' keys
    if keys.just_pressed(KeyCode::KeyM) && !cards.0 {
        ov.step(shift);
    }
    let mut turn = 0;
    if keys.just_pressed(KeyCode::KeyG) && matches!(ov.kind, Some(Overlay::Price | Overlay::Made | Overlay::Deposits)) && !cards.0 {
        turn = if shift { -1 } else { 1 };
    }
    for (i, chip) in &chips {
        if *i == Interaction::Pressed {
            // the pill, which shows while the map is plain, brings back the last overlay
            let want = if chip.0.is_none() && ov.kind.is_none() { Some(ov.last) } else { chip.0 };
            ov.set(want);
        }
    }
    for (i, s) in &steps {
        if *i == Interaction::Pressed {
            turn = s.0;
        }
    }
    for (i, c) in &ranges {
        if *i == Interaction::Pressed {
            if let Some(kind) = ov.kind {
                if ov.range(kind) != c.0 {
                    ov.ranges.insert(kind, c.0);
                }
            }
        }
    }
    if turn != 0 {
        if ov.kind == Some(Overlay::Deposits) {
            ov.deposit = (ov.deposit as i32 + turn).rem_euclid(KINDS.len() as i32) as usize;
        } else {
            let n = data.r.header.goods.len() as i32;
            // a good that has not begun by the year on the clock is stepped over (docs/rules/goods.md)
            let month = (tl.t.max(0.0).floor() as usize).min(data.r.ny.saturating_sub(1)) * 12 + 11;
            for _ in 0..n {
                ov.good = (ov.good as i32 + turn).rem_euclid(n) as usize;
                if replay::begun(&data.r, ov.good, month) {
                    break;
                }
            }
        }
    }
}

/// The ends of a view in year `k` under its range, from the years' own ends
/// kept in the cache: those the cache lacks are reckoned here when they are
/// few (a year that has just arrived) or every year must be (a still), and
/// otherwise off the main thread, the year's own ends standing in until they
/// come. Returns the scaling, and whether it is the one asked for.
#[allow(clippy::too_many_arguments)]
fn scaling_for(ov: &mut MapOverlay, data: &Data, k: usize, kind: Overlay, which: usize, range: Range, span: Option<Span>, stills: bool) -> Scaling {
    let r = &data.r;
    let key = (kind, which, span);
    let filled = r.filled.min(r.ny);
    // a reckoning that has come back: kept, and the map painted again
    if let Some((_, task)) = ov.ends.task.as_mut() {
        if let Some(done) = bevy::tasks::futures::check_ready(task) {
            let (tkey, _) = ov.ends.task.take().unwrap();
            let years = ov.ends.years.entry(tkey).or_default();
            for (j, e) in done {
                if years.len() <= j {
                    years.resize(j + 1, None);
                }
                years[j] = Some(e);
            }
            ov.ends.version += 1;
        }
    }
    // the town ground's whole run is its own scale, and needs no years
    if kind == Overlay::Growth && range == Range::Whole && span.is_none() {
        return Scaling { range, ends: None };
    }
    let want = match range {
        Range::Year => k..k + 1,
        Range::Thirty => 0..k + 1,
        Range::Whole => 0..filled.max(k + 1),
    };
    let have = |ov: &MapOverlay, j: usize| ov.ends.years.get(&key).and_then(|v| v.get(j)).is_some_and(|e| e.is_some());
    let missing: Vec<usize> = want.clone().filter(|&j| !have(ov, j)).collect();
    let keep = |ov: &mut MapOverlay, j: usize, e: Option<Ends>| {
        let years = ov.ends.years.entry(key).or_default();
        if years.len() <= j {
            years.resize(j + 1, None);
        }
        years[j] = Some(e);
    };
    if missing.len() <= INLINE_YEARS || stills {
        for &j in &missing {
            let e = ends_of(r, j, kind, which, span);
            keep(ov, j, e);
            ov.ends.inline += 1;
        }
    } else {
        // many years at once: off the main thread, on the run as it stands (a copy of a pointer)
        let busy = ov.ends.task.as_ref().is_some_and(|t| t.0 == key);
        if !busy && ov.ends.task.is_none() {
            let run = data.r.clone();
            let years = missing.clone();
            ov.ends.away += years.len() as u64;
            ov.ends.task = Some((key, bevy::tasks::AsyncComputeTaskPool::get().spawn(async move { years.into_iter().map(|j| (j, ends_of(&run, j, kind, which, span))).collect() })));
        }
        // meanwhile this year's own ends stand in
        if !have(ov, k) {
            let e = ends_of(r, k, kind, which, span);
            keep(ov, k, e);
            ov.ends.inline += 1;
        }
        let own = ov.ends.years.get(&key).and_then(|v| v.get(k)).copied().flatten().flatten();
        return Scaling { range, ends: own };
    }
    let years: Vec<Option<Ends>> = ov.ends.years.get(&key).map_or(Vec::new(), |v| v[..want.end.min(v.len())].iter().map(|e| e.flatten()).collect());
    Scaling { range, ends: ranges::ends(range, &years, k) }
}

/// Write the chosen overlay for the year on the clock into the ground's
/// texture, and turn the wash on or off. Runs only when one of them changes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint(
    mut ov: ResMut<MapOverlay>,
    tl: Res<Timeline>,
    data: Res<Data>,
    months: Res<Months>,
    live: Option<Res<crate::live::Live>>,
    stills: Option<Res<crate::ShotPlanArgs>>,
    sm: Option<Res<SeasonMats>>,
    mut mats: ResMut<Assets<terrain::GroundMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let _frames = crate::frames::span(crate::frames::Sys::MapView);
    let Some(sm) = sm else { return };
    let r = &data.r;
    let k = (tl.t.floor() as usize).min(r.ny - 1);
    let good = ov.good.min(r.header.goods.len().saturating_sub(1));
    if good != ov.good {
        ov.good = good;
    }
    // prices and wages go by the month where the clock shows one and the month is held
    let month = match ov.kind {
        Some(Overlay::Price | Overlay::Wages) => month_shown(&months, tl.t, crate::shown_date(&tl, &data, live.as_deref(), stills.is_some()).is_some()),
        _ => None,
    };
    let which = ov.kind.map_or(0, |kind| ov.which(kind));
    let range = ov.kind.filter(|k| k.scaled()).map(|k| ov.range(k));
    let span = ov.trend_shown();
    // a reckoning off the main thread that has landed moves the version, and the map is painted again
    if let Some((_, task)) = ov.ends.task.as_ref() {
        if task.is_finished() {
            if let (Some(kind), Some(range)) = (ov.kind, range) {
                scaling_for(&mut ov, &data, k, kind, which, range, span, stills.is_some());
            }
        }
    }
    // GROUND-BRUSH: the brush's preview is let go once the year after its strokes has come with them
    if !ov.preview.cells.is_empty() && r.filled > ov.preview.from + 1 {
        ov.preview = Preview::default();
    }
    // (a trend is of whole years, and takes no month)
    let month = if span.is_some() { None } else { month };
    let key = (ov.kind, which, k, r.filled, month, range, span, ov.ends.version);
    if ov.painted == Some(key) {
        return;
    }
    let month = month.map(|t| Month::of(r, &months, t));
    let w = match (ov.kind, range, span) {
        (Some(kind), Some(range), Some(span)) => {
            let scaling = scaling_for(&mut ov, &data, k, kind, which, range, Some(span), stills.is_some());
            ov.scale = None;
            Some(trend_wash(r, k, kind, which, span, scaling))
        }
        (Some(kind), Some(range), None) => {
            let scaling = scaling_for(&mut ov, &data, k, kind, which, range, None, stills.is_some());
            ov.scale = scaling.ends.map(|e| Scale::of(e, kind.spec().log));
            Some(wash(r, k, kind, which, month.as_ref(), scaling))
        }
        (Some(kind), None, _) => {
            ov.scale = None;
            Some(wash(r, k, kind, which, month.as_ref(), Scaling { range: Range::Year, ends: None }))
        }
        _ => {
            ov.scale = None;
            None
        }
    };
    let mut w = w;
    if let (Some(w), Some(Overlay::Deposits), Some(sc)) = (w.as_mut(), ov.kind, ov.scale) {
        paint_preview(&mut w.cells, &ov.preview, k, which, &sc);
    }
    let handle = mats.get(&sm.terrain).map(|m| m.extension.overlay.clone());
    if let (Some(w), Some(h)) = (&w, &handle) {
        if let Some(mut img) = images.get_mut(h) {
            img.data = Some(w.cells.clone());
        }
    }
    // the material is touched only when the wash goes on or off or changes its
    // kind; a new year of the same overlay is new texels and nothing more
    let Some(now) = mats.get(&sm.terrain).map(|m| m.extension.params) else { return };
    let want = match &w {
        Some(w) => (STRENGTH, w.soft, if w.lines { 1.0 } else { 0.0 }),
        None => (0.0, now.overlay_soft, now.overlay_lines),
    };
    if (now.overlay, now.overlay_soft, now.overlay_lines) != want {
        if let Some(mut m) = mats.get_mut(&sm.terrain) {
            let p = &mut m.extension.params;
            (p.overlay, p.overlay_soft, p.overlay_lines) = want;
        }
    }
    // the woods would hide the wash under their leaves, so on a map view the trees
    // stand aside; towns, works, roads and railways stay, being what the figures follow.
    // Nothing is hidden here: `show_trees` in main.rs reads `kind` and settles what shows
    // of the trees together with the season, which used to put the leaves back
    // (the key is taken again: the ends' version may have moved while they were reckoned)
    let which = ov.kind.map_or(0, |kind| ov.which(kind));
    ov.painted = Some((ov.kind, which, k, r.filled, month.as_ref().map(|m| m.t), range, span, ov.ends.version));
    let legend = w.map(|w| w.legend);
    if ov.legend != legend {
        ov.legend = legend;
        ov.version += 1;
    }
}

/// An empty overlay texture, for the ground material to start with.
pub fn blank(r: &Replay) -> Image {
    terrain::cell_image(r, vec![0; r.n * 4], TextureFormat::Rgba8Unorm)
}

fn legend_card(
    mut commands: Commands,
    mut ov: ResMut<MapOverlay>,
    data: Res<Data>,
    font: Res<UiFont>,
    icons: Res<crate::icons::Icons>,
    fam: Option<Res<FamilyView>>,
    sheet: Option<Res<crate::shots::Sheet>>,
    mut card: Query<(Entity, Option<&Children>, &mut Visibility), With<LegendCard>>,
) {
    let _frames = crate::frames::span(crate::frames::Sys::MapView);
    if ov.card == Some(ov.version) {
        return;
    }
    let Ok((e, children, mut vis)) = card.single_mut() else { return };
    // the standard shots are of the island: the Map views button is left out of them, as the news is
    vis.set_if_neq(if sheet.is_some() && ov.kind.is_none() { Visibility::Hidden } else { Visibility::Inherited });
    ov.card = Some(ov.version);
    if let Some(ch) = children {
        for c in ch.iter() {
            commands.entity(c).despawn();
        }
    }
    let f = font.0.clone();
    let accent = fam.as_ref().and_then(|v| v.player()).map(|p| p.colour).unwrap_or(Color::srgb(0.55, 0.42, 0.30));
    let (kind, legend, trend_on) = (ov.kind, ov.legend.clone(), ov.trend);
    let chosen = match kind {
        Some(Overlay::Deposits) => KINDS.get(ov.deposit).map(|k| k.label.to_string()).unwrap_or_default(),
        _ => data.r.header.goods.get(ov.good).map(|g| g.label.clone()).unwrap_or_default(),
    };
    commands.entity(e).with_children(|p| {
        let (Some(kind), Some(l)) = (kind, legend) else {
            // the map is plain: a pill that brings the overlays back
            p.spawn((
                Node { align_items: AlignItems::Center, column_gap: Val::Px(7.0), padding: UiRect::axes(Val::Px(9.0), Val::Px(4.0)), border_radius: BorderRadius::all(Val::Px(10.0)), ..default() },
                BackgroundColor(ui::BTN),
                Interaction::default(),
                FocusPolicy::Block,
                ui::Hoverable,
                OverlayChip(None),
            ))
            .with_children(|row| {
                row.spawn(crate::icons::icon(&icons.map, 16.0, INK));
                row.spawn(text(&f, "Map views", 13.5, INK));
                ui::keycap(row, &f, "M");
            });
            return;
        };
        p.spawn(Node { width: Val::Px(CARD_W), align_items: AlignItems::Center, column_gap: Val::Px(7.0), ..default() }).with_children(|row| {
            row.spawn(crate::icons::icon(&icons.map, 17.0, INK));
            row.spawn(text(&f, &l.title, 17.0, INK));
        });
        p.spawn((text(&f, &l.note, 12.5, MUTED), Node { width: Val::Px(CARD_W), ..default() }));
        if !l.ramp.is_empty() {
            // the scale: the tints as they lie on the map, a fine mark at each round figure,
            // the figures under their marks, and what the two ends stand for
            p.spawn(Node { width: Val::Px(CARD_W), height: Val::Px(12.0), margin: UiRect::top(Val::Px(2.0)), border_radius: BorderRadius::all(Val::Px(6.0)), overflow: Overflow::clip(), ..default() }).with_children(|bar| {
                for c in &l.ramp {
                    bar.spawn((Node { flex_grow: 1.0, ..default() }, BackgroundColor(Color::srgb_u8(c[0], c[1], c[2]))));
                }
                for (at, _) in &l.ticks {
                    bar.spawn((Node { position_type: PositionType::Absolute, left: Val::Px((at * CARD_W - 0.5).clamp(0.0, CARD_W - 1.0)), top: Val::Px(0.0), width: Val::Px(1.0), height: Val::Px(12.0), ..default() }, BackgroundColor(INK.with_alpha(0.45))));
                }
            });
            p.spawn(Node { width: Val::Px(CARD_W), height: Val::Px(15.0), ..default() }).with_children(|row| {
                for (at, words) in &l.ticks {
                    // the words centred on their mark, kept inside the card (about 6.5 px a letter)
                    let w = 6.5 * words.chars().count() as f32 + 4.0;
                    let left = (at * CARD_W - w / 2.0).clamp(0.0, CARD_W - w);
                    row.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(left), width: Val::Px(w), justify_content: JustifyContent::Center, ..default() },)).with_children(|t| {
                        t.spawn((text(&f, words, 12.0, INK), TextLayout::default().with_no_wrap()));
                    });
                }
            });
            p.spawn(Node { width: Val::Px(CARD_W), justify_content: JustifyContent::SpaceBetween, ..default() }).with_children(|ends| {
                ends.spawn(text(&f, &l.low, 11.5, MUTED));
                ends.spawn(text(&f, &l.high, 11.5, MUTED));
            });
        }
        if !l.keys.is_empty() {
            // the tints that stand for a word: a swatch and its word each
            p.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(6.0), ..default() }).with_children(|row| {
                for (c, words) in &l.keys {
                    row.spawn((Node { width: Val::Px(14.0), height: Val::Px(10.0), border_radius: BorderRadius::all(Val::Px(3.0)), ..default() }, BackgroundColor(Color::srgb_u8(c[0], c[1], c[2]))));
                    row.spawn((text(&f, words, 12.0, INK), Node { margin: UiRect::right(Val::Px(8.0)), ..default() }));
                }
            });
        }
        if let Some((rising, falling)) = &l.movers {
            // the movers: two columns of three towns, each row taking the camera there
            p.spawn(Node { width: Val::Px(CARD_W), column_gap: Val::Px(12.0), margin: UiRect::top(Val::Px(2.0)), ..default() }).with_children(|cols| {
                for (head, list, ink) in [("Rising fastest", rising, rgb(RISE[4])), ("Falling fastest", falling, rgb(FALL[4]))] {
                    cols.spawn(Node { flex_direction: FlexDirection::Column, flex_grow: 1.0, flex_basis: Val::Px(0.0), row_gap: Val::Px(1.0), ..default() }).with_children(|col| {
                        col.spawn(text(&f, head, 12.0, MUTED));
                        if list.is_empty() {
                            col.spawn(text(&f, "none", 12.5, MUTED));
                        }
                        for m in list {
                            col.spawn((
                                Node { justify_content: JustifyContent::SpaceBetween, padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(5.0)), ..default() },
                                BackgroundColor(Color::NONE),
                                Interaction::default(),
                                FocusPolicy::Block,
                                MoverRow(m.town),
                            ))
                            .with_children(|row| {
                                row.spawn((text(&f, &m.name, 13.0, INK), TextLayout::default().with_no_wrap()));
                                row.spawn(text(&f, &format!("{} a year", trend::rate_words(m.rate)), 13.0, Color::srgb_u8(ink[0], ink[1], ink[2])));
                            });
                        }
                    });
                }
            });
            let floor = ui::thousands(trend::FLOOR_PEOPLE);
            p.spawn((text(&f, &format!("Towns of {floor} people or more at the start; a click takes you there."), 11.5, MUTED), Node { width: Val::Px(CARD_W), ..default() }));
        }
        if let Some(range) = l.range {
            // how the ends are taken: three small chips, the one in use lit
            p.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(5.0), ..default() }).with_children(|row| {
                row.spawn(text(&f, "Colours from", 12.0, MUTED));
                for r in Range::ALL {
                    chip_button(row, &f, r.chip(), r == range, accent, RangeChip(r));
                }
            });
        }
        // the Trend switch, greyed on the views that have no trend
        p.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(5.0), ..default() }).with_children(|row| {
            let can = trend::trends(kind);
            row.spawn(text(&f, "Trend", 12.0, if can { MUTED } else { MUTED.with_alpha(0.45) }));
            let chips = std::iter::once((None, "Off")).chain(Span::ALL.into_iter().map(|s| (Some(s), s.chip())));
            for (s, word) in chips {
                if can {
                    chip_button(row, &f, word, s == trend_on, accent, TrendChip(s));
                } else {
                    // greyed: nothing to press
                    row.spawn((Node { padding: UiRect::axes(Val::Px(9.0), Val::Px(3.0)), border_radius: BorderRadius::all(Val::Px(10.0)), ..default() }, BackgroundColor(ui::BTN.with_alpha(0.45)))).with_children(|c| {
                        c.spawn(text(&f, word, 12.5, INK.with_alpha(0.3)));
                    });
                }
            }
        });
        if matches!(kind, Overlay::Price | Overlay::Made | Overlay::Deposits) {
            // which good: one step back or on, never a list to scroll
            p.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(8.0), ..default() }).with_children(|row| {
                ui::small_button(row, &f, "Back", GoodStep(-1));
                // as wide as the longest name there could be, so that Next stays under the pointer from good to good
                row.spawn(Node { width: Val::Px(GOOD_BOX_W), justify_content: JustifyContent::Center, overflow: Overflow::clip_x(), ..default() }).with_children(|name| {
                    name.spawn((text(&f, &chosen, 14.0, INK), TextLayout::default().with_no_wrap()));
                });
                ui::small_button(row, &f, "Next", GoodStep(1));
                ui::keycap(row, &f, "G");
            });
        }
        // (task A) nine views are more than one line of chips holds: they go on to a second
        p.spawn((Node { width: Val::Px(CARD_W), column_gap: Val::Px(5.0), row_gap: Val::Px(5.0), flex_wrap: FlexWrap::Wrap, padding: UiRect::top(Val::Px(7.0)), margin: UiRect::top(Val::Px(1.0)), border: UiRect::top(Val::Px(1.0)), align_items: AlignItems::Center, ..default() }, BorderColor::all(RULE))).with_children(|row| {
            for o in Overlay::ALL {
                chip_button(row, &f, o.chip(), o == kind, accent, OverlayChip(Some(o)));
            }
            row.spawn(Node { flex_grow: 1.0, ..default() });
            chip_button(row, &f, "Plain map", false, accent, OverlayChip(None));
        });
    });
}

#[cfg(test)]
#[path = "overlay_tests.rs"]
mod tests;
