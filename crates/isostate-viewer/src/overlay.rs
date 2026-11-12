//! Map overlays: the numbers behind the picture, washed over the ground so the
//! map can answer "why is that there". Land rent, each town's market area, how
//! far every square is from its market, what a good costs across the island,
//! and what a month's work pays.
//!
//! Each is one tint per square of the simulation's grid, in a few muted inks,
//! written to a texture the ground shader reads (terrain.wesl, `overlay_wash`).
//! A legend card says what is shown, with the real numbers at the ends of the
//! scale, and the inspector says the value in the square that was clicked.
//! Nothing is shown that the simulation does not record: the scales are the
//! year's own figures, never a guess.

// see the note at the top of ui.rs
#![allow(deprecated)]

use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::ui::FocusPolicy;

use crate::family::{chip_button, FamilyView};
use crate::replay::{self, Replay};
use crate::ui::{self, text, INK, MUTED, RULE};
use crate::{terrain, Data, SeasonMats, Timeline, UiFont};

/// How much of the ground's own colour the wash replaces.
const STRENGTH: f32 = 0.88;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Overlay {
    Rent,
    Areas,
    Reach,
    Price,
    Wages,
}

impl Overlay {
    pub const ALL: [Overlay; 5] = [Overlay::Rent, Overlay::Areas, Overlay::Reach, Overlay::Price, Overlay::Wages];

    /// The word on its chip.
    fn chip(self) -> &'static str {
        match self {
            Overlay::Rent => "Rent",
            Overlay::Areas => "Markets",
            Overlay::Reach => "Reach",
            Overlay::Price => "Prices",
            Overlay::Wages => "Wages",
        }
    }
}

/// For the command line: "rent", "areas", "reach", "price", "price:coal",
/// "wages", or "plain" for no overlay. The good is by name or label, and stays
/// as it was if none is given. None if the word is not an overlay at all.
pub fn parse(word: &str, r: &Replay) -> Option<(Option<Overlay>, Option<usize>)> {
    let w = word.to_ascii_lowercase();
    let (head, tail) = w.split_once(':').map_or((w.as_str(), None), |(a, b)| (a, Some(b)));
    let kind = match head {
        "plain" => None,
        "rent" => Some(Overlay::Rent),
        "areas" => Some(Overlay::Areas),
        "reach" => Some(Overlay::Reach),
        "price" => Some(Overlay::Price),
        "wages" => Some(Overlay::Wages),
        _ => return None,
    };
    let good = tail.and_then(|t| r.header.goods.iter().position(|g| g.name.eq_ignore_ascii_case(t) || g.label.eq_ignore_ascii_case(t)));
    Some((kind, good))
}

/// What is wrong with a word given for an overlay on the command line, if
/// anything: a name that is no overlay, or a good the run does not have. Such
/// a word is otherwise passed over (the plain map, or the good as it was).
pub fn complaint(word: &str, r: &Replay) -> Option<String> {
    match parse(word, r) {
        None => Some(format!("\"{word}\" is not a map view: use rent, areas, reach, price, price:GOOD, wages or plain")),
        Some((_, None)) => {
            let good = word.split_once(':')?.1;
            let names: Vec<&str> = r.header.goods.iter().map(|g| g.name.as_str()).collect();
            Some(format!("\"{good}\" is not a good of this run: use one of {}", names.join(", ")))
        }
        Some(_) => None,
    }
}

/// Whether a word of a `--view` names an overlay (before the run is loaded).
pub fn is_word(word: &str) -> bool {
    let w = word.to_ascii_lowercase();
    matches!(w.split(':').next().unwrap_or(""), "plain" | "rent" | "areas" | "reach" | "price" | "wages")
}

/// What is washed over the map, if anything.
#[derive(Resource)]
pub struct MapOverlay {
    pub kind: Option<Overlay>,
    /// the good whose price the price overlay shows
    pub good: usize,
    /// the overlay the pill brings back
    last: Overlay,
    /// what the texture holds: overlay, good, year index, years filled
    painted: Option<(Option<Overlay>, usize, usize, usize)>,
    legend: Option<Legend>,
    /// bumped when the legend card must be rebuilt
    version: u64,
    card: Option<u64>,
}

impl MapOverlay {
    pub fn new(kind: Option<Overlay>, good: usize) -> MapOverlay {
        MapOverlay { kind, good, last: kind.unwrap_or(Overlay::Rent), painted: None, legend: None, version: 0, card: None }
    }

    pub fn set(&mut self, kind: Option<Overlay>) {
        self.kind = kind;
        if let Some(k) = kind {
            self.last = k;
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
        matches!(self.kind, Some(Overlay::Areas | Overlay::Price | Overlay::Wages))
    }

    /// What the overlay says of a square, for the inspector.
    pub fn said(&self, r: &Replay, k: usize, c: usize) -> Option<String> {
        said(r, k, c, self.kind?, self.good)
    }
}

// ------------------------------------------------------------------ the tints

/// Five stops from low to high, in sRGB. Inks that sit on the paper: sepia for
/// rent, slate for distance, madder for prices, mulberry for wages. None is
/// the green of the fields, so a map view never reads as the land itself.
const SEPIA: [u32; 5] = [0xf4ead0, 0xe6cf9c, 0xd2aa6a, 0xb07e45, 0x80552e];
const SLATE: [u32; 5] = [0xeeeee2, 0xc9d6d2, 0x9bb3ba, 0x6d8d9e, 0x45647a];
const MADDER: [u32; 5] = [0xf5eadc, 0xe8cfbb, 0xd2aa96, 0xb28171, 0x86574d];
const MULBERRY: [u32; 5] = [0xf2ebe6, 0xdccbd2, 0xbfa3b8, 0x9a7b9b, 0x6f557c];
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
}

/// `v` to two significant figures.
pub fn round_sig(v: f32) -> f32 {
    if v <= 0.0 || !v.is_finite() {
        return 0.0;
    }
    let mag = 10f32.powf(v.log10().floor() - 1.0);
    (v / mag).round() * mag
}

/// The value a share `q` of the way up a sorted list.
fn quantile(sorted: &[f32], q: f32) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted[((sorted.len() - 1) as f32 * q).round() as usize]
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
    r.rent.get(k * r.n + c).map(|&b| 12.0 * replay::rent_of(b))
}

/// How far a square is from its market, in kilometres of flat open country:
/// the record's path cost counts one for a 2 km square of lowland with no road,
/// in quarters. None where there is no way to a market at all.
fn reach_km(r: &Replay, k: usize, c: usize) -> Option<f32> {
    r.dist.get(k * r.n + c).filter(|&&b| b < 255).map(|&b| b as f32 * 0.5)
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

// ------------------------------------------------------------------ one year's overlay

/// What the legend card says.
#[derive(Clone, Debug, PartialEq)]
pub struct Legend {
    pub title: String,
    pub note: String,
    /// the tints from low to high; empty where there is no scale
    pub ramp: Vec<[u8; 3]>,
    /// the figures at the two ends, in words
    pub low: String,
    pub high: String,
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

/// The overlay for year index `k`: tints, and the legend that explains them.
pub fn wash(r: &Replay, k: usize, kind: Overlay, good: usize) -> Wash {
    let year = r.y0 + k as i32;
    let mut cells = vec![0u8; r.n * 4];
    // every land cell carries its market as its region, so the shader can rule the boundaries
    let region = |c: usize| market(r, k, c).map_or(255, |t| (t + 1).min(254) as u8);
    let mut put = |c: usize, col: [u8; 3]| cells[c * 4..c * 4 + 4].copy_from_slice(&[col[0], col[1], col[2], region(c)]);
    let lands = (0..r.n).filter(|&c| land(r, c));
    let none = |title: String, note: String| Legend { title, note, ramp: Vec::new(), low: String::new(), high: String::new() };
    match kind {
        Overlay::Rent => {
            let mut vals: Vec<f32> = lands.clone().filter_map(|c| rent_year(r, k, c)).filter(|&v| v > 0.0).collect();
            vals.sort_by(f32::total_cmp);
            let title = format!("Land rent in {year}");
            if vals.is_empty() {
                return Wash { cells, legend: none(title, "This record does not hold the rent of the land.".into()), soft: 0.5, lines: false };
            }
            // rents run over several powers of ten, so the scale goes by ratio;
            // its ends leave out the poorest tenth and the dearest twentieth (the town
            // centres, which would otherwise squeeze the farmland into two tints)
            let sc = Scale { lo: round_sig(quantile(&vals, 0.10)), hi: round_sig(quantile(&vals, 0.95)), log: true };
            for c in lands {
                let v = rent_year(r, k, c).unwrap_or(0.0);
                put(c, ramp(&SEPIA, sc.at(v)));
            }
            let legend = Legend {
                title,
                note: "What the ground of each square earns its owners in a year, from farms, mines, works and town houses.".into(),
                ramp: ramp_bar(&SEPIA),
                low: format!("{} or less", about(sc.lo)),
                high: format!("{} or more", about(sc.hi)),
            };
            Wash { cells, legend, soft: 0.5, lines: false }
        }
        Overlay::Reach => {
            let mut vals: Vec<f32> = lands.clone().filter_map(|c| reach_km(r, k, c)).collect();
            vals.sort_by(f32::total_cmp);
            let title = format!("The way to market in {year}");
            if r.dist.is_empty() || vals.is_empty() {
                return Wash { cells, legend: none(title, "This record does not hold the way to market.".into()), soft: 0.5, lines: false };
            }
            // the far end is a round number of kilometres that nine tenths of the land is within
            let hi = ((quantile(&vals, 0.90) / 5.0).ceil() * 5.0).max(5.0);
            let sc = Scale { lo: 0.0, hi, log: false };
            for c in lands {
                // land with no way to a market is as far as can be
                put(c, ramp(&SLATE, reach_km(r, k, c).map_or(1.0, |v| sc.at(v))));
            }
            let legend = Legend {
                title,
                note: "How far each square is from the market it trades in, counted as kilometres of flat open country. Roads, rivers, the sea and railways shorten the way; hills and woods lengthen it.".into(),
                ramp: ramp_bar(&SLATE),
                low: "at the market".into(),
                high: format!("{hi:.0} km or more"),
            };
            Wash { cells, legend, soft: 0.5, lines: false }
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
            let is_price = kind == Overlay::Price;
            let label = r.header.goods.get(good).map_or("goods".to_string(), |g| g.label.to_lowercase());
            let value = |t: usize| if is_price { price(r, k, t, good) } else { wage(r, k, t) };
            let title = if is_price { format!("The price of {label} in {year}") } else { format!("Wages in {year}") };
            // the cheapest and the dearest market are the ends of the scale
            let towns: Vec<(usize, f32)> = markets(r, k).into_iter().filter_map(|t| value(t).map(|v| (t, v))).collect();
            let lo = towns.iter().copied().min_by(|a, b| a.1.total_cmp(&b.1));
            let hi = towns.iter().copied().max_by(|a, b| a.1.total_cmp(&b.1));
            let (Some(lo), Some(hi)) = (lo, hi) else {
                for c in lands {
                    put(c, rgb(BLANK));
                }
                let note = if !is_price {
                    "This record does not hold the towns' wages.".to_string()
                } else if r.town_price.is_empty() {
                    "This record does not hold the towns' prices.".to_string()
                } else {
                    format!("No market has a price for {label} in {year}.")
                };
                return Wash { cells, legend: none(title, note), soft: 0.06, lines: true };
            };
            let sc = Scale { lo: lo.1, hi: hi.1, log: false };
            let stops = if is_price { &MADDER } else { &MULBERRY };
            for c in lands {
                put(c, market(r, k, c).and_then(value).map_or(rgb(BLANK), |v| ramp(stops, sc.at(v))));
            }
            let legend = Legend {
                title,
                note: if is_price {
                    "Each square takes the price in the market it trades in, the year's average. Carriage to and from the market is not counted.".into()
                } else {
                    "What a month's work pays in each town, the year's average, shown across the town's market area.".into()
                },
                ramp: ramp_bar(stops),
                low: format!("{} at {}", coin(lo.1), town_name(r, lo.0)),
                high: format!("{} at {}", coin(hi.1), town_name(r, hi.0)),
            };
            Wash { cells, legend, soft: 0.06, lines: true }
        }
    }
}

/// What the overlay says of square `c` in year index `k`, as a sentence for the
/// inspector. None where it has nothing to say (the sea, or no record).
pub fn said(r: &Replay, k: usize, c: usize, kind: Overlay, good: usize) -> Option<String> {
    if c >= r.n || !land(r, c) {
        return None;
    }
    let town = market(r, k, c);
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
            match town.map(|t| (t, price(r, k, t, good))) {
                Some((t, Some(p))) => format!("Price: {} for {} at {}, the market this square trades in.", coin(p), g.label.to_lowercase(), town_name(r, t)),
                Some((t, None)) => format!("Price: {}'s market has no price for {} this year.", town_name(r, t), g.label.to_lowercase()),
                None => "Price: it trades in no town's market.".to_string(),
            }
        }
        Overlay::Wages => match town.and_then(|t| wage(r, k, t).map(|w| (t, w))) {
            Some((t, w)) => format!("Wages: a month's work pays {} at {}.", coin(w), town_name(r, t)),
            None => return None,
        },
    })
}

// ------------------------------------------------------------------ the legend card

#[derive(Component)]
struct LegendCard;

/// Choose an overlay (or none).
#[derive(Component, Clone, Copy)]
pub struct OverlayChip(pub Option<Overlay>);

/// The good before or after the one whose price is mapped.
#[derive(Component, Clone, Copy)]
pub struct GoodStep(pub i32);

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
/// goods of the price overlay; the chips on the card do the same by hand.
fn input(
    keys: Res<ButtonInput<KeyCode>>,
    data: Res<Data>,
    mut ov: ResMut<MapOverlay>,
    chips: Query<(&Interaction, &OverlayChip), Changed<Interaction>>,
    steps: Query<(&Interaction, &GoodStep), Changed<Interaction>>,
    cards: Res<crate::game::CardOpen>,
) {
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    // while a card wants an answer the keys are the card's, as with the panels' keys
    if keys.just_pressed(KeyCode::KeyM) && !cards.0 {
        ov.step(shift);
    }
    let mut turn = 0;
    if keys.just_pressed(KeyCode::KeyG) && ov.kind == Some(Overlay::Price) && !cards.0 {
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
    let n = data.r.header.goods.len() as i32;
    if turn != 0 && n > 0 {
        ov.good = (ov.good as i32 + turn).rem_euclid(n) as usize;
    }
}

/// Write the chosen overlay for the year on the clock into the ground's
/// texture, and turn the wash on or off. Runs only when one of them changes.
fn paint(
    mut ov: ResMut<MapOverlay>,
    tl: Res<Timeline>,
    data: Res<Data>,
    sm: Option<Res<SeasonMats>>,
    mut mats: ResMut<Assets<terrain::GroundMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(sm) = sm else { return };
    let r = &data.r;
    let k = (tl.t.floor() as usize).min(r.ny - 1);
    let good = ov.good.min(r.header.goods.len().saturating_sub(1));
    let key = (ov.kind, good, k, r.filled);
    if ov.painted == Some(key) {
        return;
    }
    let w = ov.kind.map(|kind| wash(r, k, kind, good));
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
    ov.painted = Some(key);
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
    let (kind, legend) = (ov.kind, ov.legend.clone());
    let good = data.r.header.goods.get(ov.good).map(|g| g.label.clone()).unwrap_or_default();
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
        p.spawn(Node { width: Val::Px(396.0), align_items: AlignItems::Center, column_gap: Val::Px(7.0), ..default() }).with_children(|row| {
            row.spawn(crate::icons::icon(&icons.map, 17.0, INK));
            row.spawn(text(&f, &l.title, 17.0, INK));
        });
        p.spawn((text(&f, &l.note, 12.5, MUTED), Node { width: Val::Px(396.0), ..default() }));
        if !l.ramp.is_empty() {
            // the scale: the tints as they lie on the map, and the figures at its two ends
            p.spawn(Node { width: Val::Px(396.0), height: Val::Px(12.0), margin: UiRect::top(Val::Px(2.0)), border_radius: BorderRadius::all(Val::Px(6.0)), overflow: Overflow::clip(), ..default() }).with_children(|bar| {
                for c in &l.ramp {
                    bar.spawn((Node { flex_grow: 1.0, ..default() }, BackgroundColor(Color::srgb_u8(c[0], c[1], c[2]))));
                }
            });
            p.spawn(Node { width: Val::Px(396.0), justify_content: JustifyContent::SpaceBetween, ..default() }).with_children(|ends| {
                for (word, figure, end) in [("low", &l.low, AlignItems::Start), ("high", &l.high, AlignItems::End)] {
                    ends.spawn(Node { flex_direction: FlexDirection::Column, align_items: end, ..default() }).with_children(|col| {
                        col.spawn(text(&f, word, 11.5, MUTED));
                        col.spawn(text(&f, figure, 13.0, INK));
                    });
                }
            });
        }
        if kind == Overlay::Price {
            // which good: one step back or on, never a list to scroll
            p.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(8.0), ..default() }).with_children(|row| {
                ui::small_button(row, &f, "Back", GoodStep(-1));
                row.spawn(Node { min_width: Val::Px(84.0), justify_content: JustifyContent::Center, ..default() }).with_children(|name| {
                    name.spawn(text(&f, &good, 14.0, INK));
                });
                ui::small_button(row, &f, "Next", GoodStep(1));
                ui::keycap(row, &f, "G");
            });
        }
        p.spawn((Node { width: Val::Px(396.0), column_gap: Val::Px(5.0), padding: UiRect::top(Val::Px(7.0)), margin: UiRect::top(Val::Px(1.0)), border: UiRect::top(Val::Px(1.0)), align_items: AlignItems::Center, ..default() }, BorderColor::all(RULE))).with_children(|row| {
            for o in Overlay::ALL {
                chip_button(row, &f, o.chip(), o == kind, accent, OverlayChip(Some(o)));
            }
            row.spawn(Node { flex_grow: 1.0, ..default() });
            chip_button(row, &f, "Plain map", false, accent, OverlayChip(None));
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::{Good, Header, Meta, Town};

    /// A strip of four squares in one year: sea, then two squares that trade at
    /// Ashby (which stands on the first of them), then one that trades at Brill.
    fn strip() -> Replay {
        let town = |name: &str, node: usize| Town { name: name.into(), node, founded: 1700 };
        let good = |name: &str, label: &str| Good { name: name.into(), label: label.into() };
        let header = Header {
            meta: Meta { label: String::new(), lite: false, seed: 1, w: 4, h: 1, start_year: 1750, n_years: 1 },
            goods: vec![good("grain", "Grain"), good("coal", "Coal")],
            methods: Vec::new(),
            towns: vec![town("Ashby", 1), town("Brill", 3)],
            firms: Vec::new(),
            events: Vec::new(),
            disp: Vec::new(),
            uses: Vec::new(),
            arrays: Vec::new(),
            series: Default::default(),
        };
        Replay {
            header,
            w: 4,
            h: 1,
            n: 4,
            ny: 1,
            y0: 1750,
            n_towns: 2,
            elev: vec![0; 4],
            terrain: vec![0, 1, 1, 2],
            river: vec![0; 4],
            down: vec![-1; 4],
            use_: vec![0; 4],
            inten: vec![0; 4],
            trans: vec![0; 4],
            catch: vec![0, 1, 1, 2],
            // a month's rent of 0.001, 1 and 1000 on the record's scale; none for the sea
            rent: vec![0, 1, 128, 255],
            // quarter units of path cost: at the market, 3 units away, and no way at all
            dist: vec![0, 0, 12, 255],
            mix_a: vec![0; 4],
            mix_b: vec![0; 4],
            mix_s: vec![255; 4],
            town_active: vec![1, 1],
            town_pop: vec![100.0, 50.0],
            town_urban: vec![10.0, 5.0],
            town_wage: vec![2.0, 3.0],
            // grain in both towns; coal only at Brill
            town_price: vec![1.0, 0.0, 1.5, 0.8],
            firm_first: Vec::new(),
            firm_len: Vec::new(),
            firm_off: Vec::new(),
            firm_k: Vec::new(),
            series_names: Vec::new(),
            series: Vec::new(),
            series_len: 12,
            filled: 1,
            owner: Vec::new(),
            workings: Vec::new(),
        }
    }

    fn tint(w: &Wash, c: usize) -> [u8; 4] {
        [w.cells[c * 4], w.cells[c * 4 + 1], w.cells[c * 4 + 2], w.cells[c * 4 + 3]]
    }

    #[test]
    fn the_sea_is_left_alone_and_land_carries_its_market() {
        let r = strip();
        for kind in Overlay::ALL {
            let w = wash(&r, 0, kind, 0);
            assert_eq!(tint(&w, 0), [0, 0, 0, 0], "{kind:?} tints the sea");
            assert_eq!((tint(&w, 1)[3], tint(&w, 2)[3], tint(&w, 3)[3]), (1, 1, 2), "{kind:?} regions");
        }
    }

    #[test]
    fn prices_and_wages_run_from_the_cheapest_town_to_the_dearest() {
        let r = strip();
        let w = wash(&r, 0, Overlay::Price, 0);
        assert_eq!(w.legend.low, "\u{00a3}1.00 at Ashby");
        assert_eq!(w.legend.high, "\u{00a3}1.50 at Brill");
        assert_eq!(&tint(&w, 1)[..3], &rgb(MADDER[0]));
        assert_eq!(&tint(&w, 3)[..3], &rgb(MADDER[4]));
        assert!(w.lines);
        // coal has a price only at Brill: Ashby's squares are blank, and the inspector says so
        let w = wash(&r, 0, Overlay::Price, 1);
        assert_eq!(&tint(&w, 1)[..3], &rgb(BLANK));
        assert_eq!(said(&r, 0, 2, Overlay::Price, 1).unwrap(), "Price: Ashby's market has no price for coal this year.");
        assert_eq!(said(&r, 0, 3, Overlay::Price, 1).unwrap(), "Price: \u{00a3}0.80 for coal at Brill, the market this square trades in.");
        // a square outside every market area has no price to take
        let mut lone = strip();
        lone.catch[3] = 0;
        assert_eq!(said(&lone, 0, 3, Overlay::Price, 0).unwrap(), "Price: it trades in no town's market.");
        let w = wash(&r, 0, Overlay::Wages, 0);
        assert_eq!((w.legend.low.as_str(), w.legend.high.as_str()), ("\u{00a3}2.00 at Ashby", "\u{00a3}3.00 at Brill"));
        assert_eq!(said(&r, 0, 2, Overlay::Wages, 0).unwrap(), "Wages: a month's work pays \u{00a3}2.00 at Ashby.");
    }

    #[test]
    fn the_inspector_says_what_the_overlay_shows() {
        let r = strip();
        assert_eq!(said(&r, 0, 0, Overlay::Rent, 0), None);
        // 12 months of the record's middle value: 10^(128/255*6 - 3) is about 1.03 a month
        assert_eq!(said(&r, 0, 2, Overlay::Rent, 0).unwrap(), "Rent: the ground of this square earns about \u{00a3}12 a year.");
        // the record's top step is a floor, not a figure
        assert_eq!(said(&r, 0, 3, Overlay::Rent, 0).unwrap(), "Rent: the ground of this square earns \u{00a3}12k a year or more.");
        assert_eq!(said(&r, 0, 2, Overlay::Areas, 0).unwrap(), "Market: it buys and sells at Ashby, one of 2 squares that do.");
        assert_eq!(said(&r, 0, 1, Overlay::Reach, 0).unwrap(), "Way to market: Ashby's market is here.");
        assert_eq!(said(&r, 0, 2, Overlay::Reach, 0).unwrap(), "Way to market: Ashby is as far as 6.0 km of flat open country.");
        assert_eq!(said(&r, 0, 3, Overlay::Reach, 0).unwrap(), "Way to market: there is no way from here to any market.");
    }

    #[test]
    fn neighbouring_markets_never_share_a_tint() {
        let r = strip();
        let t = area_tints(&r, 0);
        assert_ne!(t[0], t[1]);
        let w = wash(&r, 0, Overlay::Areas, 0);
        assert_ne!(&tint(&w, 2)[..3], &tint(&w, 3)[..3]);
        // the year is in the title, and is not said again
        assert_eq!(w.legend.title, "Market areas in 1750");
        assert!(w.legend.note.ends_with("gives way to the next. 2 markets."));
    }

    #[test]
    fn the_key_steps_through_every_overlay_and_back_to_the_plain_map() {
        let mut ov = MapOverlay::new(None, 0);
        let mut seen = Vec::new();
        for _ in 0..6 {
            ov.step(false);
            seen.push(ov.kind);
        }
        assert_eq!(seen, vec![Some(Overlay::Rent), Some(Overlay::Areas), Some(Overlay::Reach), Some(Overlay::Price), Some(Overlay::Wages), None]);
        // backwards from the plain map is the last of them, and the pill remembers it
        ov.step(true);
        assert_eq!(ov.kind, Some(Overlay::Wages));
        ov.set(None);
        assert_eq!(ov.last, Overlay::Wages);
    }

    #[test]
    fn overlays_are_named_on_the_command_line() {
        let r = strip();
        assert_eq!(parse("rent", &r), Some((Some(Overlay::Rent), None)));
        assert_eq!(parse("price:Coal", &r), Some((Some(Overlay::Price), Some(1))));
        assert_eq!(parse("plain", &r), Some((None, None)));
        assert_eq!(parse("markets", &r), None);
        assert!(is_word("price:coal") && is_word("areas") && !is_word("markets") && !is_word("island"));
        // a word that would be passed over is complained of, a good one is not
        assert_eq!(complaint("price:coal", &r), None);
        assert_eq!(complaint("wages", &r), None);
        assert!(complaint("rents", &r).unwrap().contains("not a map view"));
        assert_eq!(complaint("price:gin", &r).unwrap(), "\"gin\" is not a good of this run: use one of grain, coal");
    }

    #[test]
    fn scales_run_from_low_to_high() {
        let even = Scale { lo: 2.0, hi: 6.0, log: false };
        assert_eq!(even.at(1.0), 0.0);
        assert_eq!(even.at(4.0), 0.5);
        assert_eq!(even.at(9.0), 1.0);
        let ratio = Scale { lo: 1.0, hi: 100.0, log: true };
        assert!((ratio.at(10.0) - 0.5).abs() < 1e-6);
        assert_eq!(ratio.at(0.0), 0.0);
        // one town, or every town alike: the middle of the ramp, not a division by nothing
        assert_eq!(Scale { lo: 3.0, hi: 3.0, log: false }.at(3.0), 0.5);
    }

    #[test]
    fn figures_are_rounded_and_written_plainly() {
        assert_eq!(round_sig(1234.0), 1200.0);
        assert!((round_sig(0.0456) - 0.046).abs() < 1e-6);
        assert_eq!(coin(1.2), "\u{00a3}1.20");
        assert_eq!(coin(12.34), "\u{00a3}12.3");
        assert_eq!(coin(480.0), "\u{00a3}480");
        assert_eq!(coin(2600.0), "\u{00a3}2,600");
        // a rough sum keeps its two figures and no more
        assert_eq!(about(97.3), "\u{00a3}97");
        assert_eq!(about(0.744), "\u{00a3}0.74");
        assert_eq!(about(4.38), "\u{00a3}4.40");
        assert_eq!(about(1234.0), "\u{00a3}1,200");
    }

    #[test]
    fn the_ramp_ends_on_its_end_stops() {
        assert_eq!(ramp(&SEPIA, 0.0), rgb(SEPIA[0]));
        assert_eq!(ramp(&SEPIA, 1.0), rgb(SEPIA[4]));
        assert_eq!(ramp(&SEPIA, 0.5), rgb(SEPIA[2]));
    }
}
