//! The Trend switch of the map views: a view coloured by how its figure has
//! changed over the last 5, 10 or 30 years, as a rate in per cent a year, and
//! the towns that rose and fell fastest by it.
//!
//! This file is the reckoning alone, kept apart from the painting: what each
//! square and each town held at the span's two ends (`figures`), the rate
//! between them (`rate`, `trend_map`), where the scale's ends stand in a year
//! (`trend_ends`) and the movers (`movers`). `overlay::trend_wash` paints it,
//! one tint to a square. The map views are to be painted otherwise before
//! long; whatever paints them then can take `TrendMap` and `movers` as they are.
//!
//! Where the years come from. A recorded run holds every year it played, and
//! a live game every year since 1750 in the same arrays (`live::apply_year`
//! writes each year as it closes; a game gone on with from a save plays its
//! years again), so the span's first year is held whenever the clock is past
//! it. A trend is of whole years: the months a live game keeps of its prices
//! and wages (`live::Months`) are not used. Before the span's first year can
//! be had (1760 for ten years) the view says so and paints nothing.

use super::*;

/// How many years a trend looks back over.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Span {
    Five,
    Ten,
    Thirty,
}

impl Span {
    pub const ALL: [Span; 3] = [Span::Five, Span::Ten, Span::Thirty];

    pub fn years(self) -> usize {
        match self {
            Span::Five => 5,
            Span::Ten => 10,
            Span::Thirty => 30,
        }
    }

    /// The word on its chip.
    pub fn chip(self) -> &'static str {
        match self {
            Span::Five => "5 years",
            Span::Ten => "10 years",
            Span::Thirty => "30 years",
        }
    }

    /// Its word on the command line and in scripts, after `trend` (`residents+trend10`, `trend:10`).
    pub fn word(self) -> &'static str {
        match self {
            Span::Five => "5",
            Span::Ten => "10",
            Span::Thirty => "30",
        }
    }

    pub fn parse(w: &str) -> Option<Span> {
        Span::ALL.into_iter().find(|s| s.word() == w.trim())
    }
}

/// The trend's setting as a word after a view (`+trend10`, `+trendoff`):
/// Some(None) for off, None if the word is not one.
pub fn parse_word(w: &str) -> Option<Option<Span>> {
    let rest = w.strip_prefix("trend")?;
    if rest == "off" {
        return Some(None);
    }
    Span::parse(rest).map(Some)
}

/// Whether a view has a trend: those that colour by an amount that can rise
/// and fall in proportion. The market areas are no amount; the way to market
/// and the deposits are, but their change is a road or a pit, which the plain
/// view shows; when town ground joined does not change. The traffic's ribbons
/// are not coloured by change (the build notes say why).
pub fn trends(kind: Overlay) -> bool {
    matches!(kind, Overlay::Residents | Overlay::Wages | Overlay::Rent | Overlay::Price | Overlay::Made)
}

/// Under this, the scale's ends never stand: a calm island, whose figures
/// move a few tenths of a per cent a year, is painted pale and not in loud
/// colours stretched over its noise. In per cent a year.
pub const LEAST_END: f32 = 0.5;

/// The movers are towns of at least this many people living in them at the
/// span's start, so that a hamlet's doubling does not lead, nor a town
/// founded a few years before (island 7's Oakbury, 383 people in 1890 and
/// 3,982 in 1900); a town that empties is among them, being one that was a
/// town (the build notes, "The floor").
pub const FLOOR_PEOPLE: f32 = 2000.0;

/// How many towns the movers name each way.
pub const MOVERS: usize = 3;

/// What became of one square or one town over the span.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Change {
    /// nothing there at either end
    Nothing,
    /// in per cent a year
    Rate(f32),
    /// nothing there at the span's start, something now
    New,
    /// something there at the span's start, nothing now
    Gone,
}

/// The rate from `a` to `b` over `years`, in per cent a year, compounded: a
/// figure that doubles in ten years reads 7.2. A figure of nothing (none, or
/// nought) at one end is new or gone, never an endless rate.
pub fn rate(a: Option<f32>, b: Option<f32>, years: usize) -> Change {
    let some = |v: Option<f32>| v.filter(|v| v.is_finite() && *v > 0.0);
    match (some(a), some(b)) {
        (Some(a), Some(b)) => Change::Rate(((b / a).powf(1.0 / years.max(1) as f32) - 1.0) * 100.0),
        (None, Some(_)) => Change::New,
        (Some(_), None) => Change::Gone,
        (None, None) => Change::Nothing,
    }
}

/// One view's figures in one year, in its own measure.
pub struct Figures {
    /// each square's figure, where the view goes square by square (rent,
    /// residents, a good of the land); empty where it goes by towns
    pub cells: Vec<Option<f32>>,
    /// each town's figure: its own (wages, prices, what it made), or its
    /// squares' together (residents, all of them; rent, the mean of its market's)
    pub towns: Vec<Option<f32>>,
}

/// The figures of year `k` a view's trend is reckoned from. None where the
/// view has no trend or the year shows nothing.
pub fn figures(r: &Replay, k: usize, kind: Overlay, which: usize) -> Option<Figures> {
    if k >= r.filled.min(r.ny) {
        return None;
    }
    let nt = r.header.towns.len();
    let lands = || (0..r.n).filter(|&c| land(r, c));
    match kind {
        Overlay::Residents => {
            if !r.kept() {
                return None;
            }
            let mut cells = vec![None; r.n];
            let mut towns = vec![0.0f32; nt];
            for c in lands().filter(|&c| r.joined(k, c).is_some()) {
                let v = r.residents(k, c);
                if v > 0.0 {
                    cells[c] = Some(v);
                    if let Some(t) = market(r, k, c) {
                        towns[t] += v;
                    }
                }
            }
            Some(Figures { cells, towns: towns.into_iter().map(Some).collect() })
        }
        Overlay::Rent => {
            if r.rent.is_empty() {
                return None;
            }
            let mut cells = vec![None; r.n];
            let mut sum = vec![(0.0f32, 0usize); nt];
            for c in lands() {
                let Some(v) = rent_year(r, k, c).filter(|&v| v > 0.0) else { continue };
                cells[c] = Some(v);
                if let Some(t) = market(r, k, c) {
                    sum[t].0 += v;
                    sum[t].1 += 1;
                }
            }
            Some(Figures { cells, towns: sum.into_iter().map(|(s, n)| (n > 0).then(|| s / n as f32)).collect() })
        }
        Overlay::Wages | Overlay::Price => {
            if !(if kind == Overlay::Wages { !r.town_wage.is_empty() } else { !r.town_price.is_empty() && made(r, k, which, None) }) {
                return None;
            }
            let mut towns = vec![None; nt];
            for t in markets(r, k) {
                towns[t] = if kind == Overlay::Wages { wage(r, k, t) } else { price(r, k, t, which) };
            }
            Some(Figures { cells: Vec::new(), towns })
        }
        Overlay::Made => {
            let m = made_map(r, k, which)?;
            let mut towns: Vec<Option<f32>> = m.towns.iter().map(|&v| Some(v)).collect();
            towns.resize(nt, None);
            let cells = if m.by_land { m.cells.into_iter().map(Some).collect() } else { Vec::new() };
            Some(Figures { cells, towns })
        }
        _ => None,
    }
}

/// What became of each square and each town of a view over the span to year `k`.
#[derive(Clone, Debug, PartialEq)]
pub struct TrendMap {
    pub span: Span,
    /// the span's first year, as an index
    pub from: usize,
    /// a change for every square: its own where the view goes square by
    /// square, else its market town's (a good of the works: on the town's own
    /// ground only); Nothing at sea
    pub cells: Vec<Change>,
    pub towns: Vec<Change>,
    /// whether the view goes town by town, and so its scale is the towns'
    pub by_town: bool,
}

/// The trend of a view over `span` to year `k`. None before the span's first
/// year is held, or where the view has no trend or shows nothing in year `k`.
pub fn trend_map(r: &Replay, k: usize, kind: Overlay, which: usize, span: Span) -> Option<TrendMap> {
    if !trends(kind) {
        return None;
    }
    let from = k.checked_sub(span.years())?;
    let now = figures(r, k, kind, which)?;
    // a good not made at the span's start has no figures then: every town's making of it is new
    let then = figures(r, from, kind, which).unwrap_or(Figures { cells: vec![None; now.cells.len()], towns: vec![None; now.towns.len()] });
    let n = span.years();
    let towns: Vec<Change> = (0..now.towns.len()).map(|t| rate(then.towns.get(t).copied().flatten(), now.towns[t], n)).collect();
    let by_town = now.cells.is_empty();
    let mut cells = vec![Change::Nothing; r.n];
    if by_town {
        let works = kind == Overlay::Made;
        for c in (0..r.n).filter(|&c| land(r, c)) {
            let Some(t) = market(r, k, c) else { continue };
            // a good of the works is each town's over its own ground, as the plain view lays it
            if works && !(town_ground(r, k, c) || r.header.towns.get(t).is_some_and(|tw| tw.node == c)) {
                continue;
            }
            cells[c] = towns.get(t).copied().unwrap_or(Change::Nothing);
        }
    } else {
        for c in (0..r.n).filter(|&c| land(r, c)) {
            cells[c] = rate(then.cells.get(c).copied().flatten(), now.cells[c], n);
        }
    }
    Some(TrendMap { span, from, cells, towns, by_town })
}

impl TrendMap {
    /// The rates the scale is taken over: the towns' where the view goes by
    /// towns, the squares' where it goes by squares.
    pub fn rates(&self) -> Vec<f32> {
        let of = if self.by_town { &self.towns } else { &self.cells };
        of.iter().filter_map(|c| if let Change::Rate(v) = c { Some(*v) } else { None }).collect()
    }

    /// How many squares are new and gone.
    pub fn new_and_gone(&self) -> (usize, usize) {
        let of = if self.by_town { &self.towns } else { &self.cells };
        (of.iter().filter(|c| **c == Change::New).count(), of.iter().filter(|c| **c == Change::Gone).count())
    }
}

/// Year `k`'s own ends of a view's trend: even about nought, each as far out
/// as the rate the twentieth part of the squares or towns lie beyond on either
/// side together (`ranges::trimmed`), never nearer than `LEAST_END`.
pub fn trend_ends(r: &Replay, k: usize, kind: Overlay, which: usize, span: Span) -> Option<Ends> {
    let m = trend_map(r, k, kind, which, span)?;
    let mut size: Vec<f32> = m.rates().into_iter().map(f32::abs).collect();
    size.sort_by(f32::total_cmp);
    let b = ranges::trimmed(&size).map_or(LEAST_END, |(_, hi)| hi.max(LEAST_END));
    Some(Ends { lo: -b, hi: b })
}

/// A town that rose or fell, with its rate in per cent a year.
#[derive(Clone, Debug, PartialEq)]
pub struct Mover {
    pub town: usize,
    pub name: String,
    pub rate: f32,
}

/// The towns rising fastest and falling fastest by the view's own measure,
/// `MOVERS` each way, of those with `FLOOR_PEOPLE` living in them at the
/// span's start. A town new in the span, or gone, has no rate and is not
/// among them.
pub fn movers(r: &Replay, m: &TrendMap) -> (Vec<Mover>, Vec<Mover>) {
    let people = |j: usize, t: usize| r.town_urban.get(j * r.n_towns + t).copied().filter(|v| v.is_finite()).unwrap_or(0.0);
    let mut rated: Vec<(usize, f32)> = (0..m.towns.len())
        .filter(|&t| people(m.from, t) >= FLOOR_PEOPLE)
        .filter_map(|t| if let Change::Rate(v) = m.towns[t] { Some((t, v)) } else { None })
        .collect();
    rated.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    let mover = |&(t, v): &(usize, f32)| Mover { town: t, name: town_name(r, t).to_string(), rate: v };
    let rising: Vec<Mover> = rated.iter().filter(|x| x.1 > 0.0).take(MOVERS).map(mover).collect();
    let falling: Vec<Mover> = rated.iter().rev().filter(|x| x.1 < 0.0).take(MOVERS).map(mover).collect();
    (rising, falling)
}

/// A rate in words: "+7.2%", "-0.4%", "+5%", "0%".
pub fn rate_words(v: f32) -> String {
    if v.abs() < 0.05 {
        "0%".to_string()
    } else if v.abs() >= 99.5 {
        format!("{v:+.0}%")
    } else if v.abs() < 0.995 {
        // a quarter per cent on a calm scale's marks is written as it is
        format!("{v:+.2}").trim_end_matches('0').to_string() + "%"
    } else {
        format!("{v:+.1}").trim_end_matches(".0").to_string() + "%"
    }
}
