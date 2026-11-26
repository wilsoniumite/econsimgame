//! The game interface: a top bar with the family, the date and the speed; a
//! tabbed side panel (family, holdings, markets, news); and a timeline along
//! the bottom with the year's news marked on it. In a sandbox the family's card
//! is the island's, the tabs are hand, rules, markets, news and families, and
//! the hand's ink is the accent (docs/designs/sandbox.md, 4.1).
//!
//! Everything is drawn in the same paper-and-ink style as the labels: warm
//! cards with rounded corners and soft shadows, Fredoka for type.

// Bevy 0.20 deprecates Interaction (and hides the old Button) in favour of ui_widgets and picking.
// Interaction plus FocusPolicy::Block is all a button needs; moving over is a job for a later upgrade.
#![allow(deprecated)]

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition, ScrollPosition};

use crate::clock;
use crate::family::{self, FamilyView};
use crate::replay::{self, Replay};
use crate::{Data, Timeline, UiFont};

pub const PAPER: Color = Color::srgba(0.99, 0.97, 0.92, 0.95);
pub const PAPER_2: Color = Color::srgb(0.965, 0.935, 0.87);
pub const INK: Color = Color::srgb(0.20, 0.17, 0.14);
pub const MUTED: Color = Color::srgb(0.47, 0.42, 0.36);
pub const RULE: Color = Color::srgb(0.89, 0.85, 0.77);
pub const BTN: Color = Color::srgb(0.945, 0.905, 0.82);
pub const BTN_HOVER: Color = Color::srgb(0.915, 0.86, 0.75);
pub const BTN_DOWN: Color = Color::srgb(0.86, 0.79, 0.65);
pub const GOOD: Color = Color::srgb(0.27, 0.52, 0.30);
pub const BAD: Color = Color::srgb(0.72, 0.28, 0.24);
/// prices: warm for dearer than 1750, cool for cheaper; neither is good or bad
pub const DEARER: Color = Color::srgb(0.80, 0.47, 0.22);
pub const CHEAPER: Color = Color::srgb(0.24, 0.47, 0.66);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Family,
    Holdings,
    Markets,
    News,
    /// a sandbox's: everything done by hand (hand_tab.rs)
    Hand,
    /// a sandbox's: the island's rules (rulebook.rs)
    Rules,
}

impl Tab {
    /// For the command line: "markets", "news", ... or anything else for no panel.
    /// "families" is the Family tab, which is what a sandbox calls it.
    pub fn parse(s: &str) -> Option<Tab> {
        match s.to_ascii_lowercase().as_str() {
            "family" | "families" => Some(Tab::Family),
            "holdings" => Some(Tab::Holdings),
            "markets" => Some(Tab::Markets),
            "news" => Some(Tab::News),
            "hand" => Some(Tab::Hand),
            "rules" => Some(Tab::Rules),
            _ => None,
        }
    }

    /// The tab's word; in a sandbox the Family tab is the Families, a place to
    /// look at the six of them and no longer the player's own.
    fn label(self, sandbox: bool) -> &'static str {
        match self {
            Tab::Family if sandbox => "Families",
            Tab::Family => "Family",
            Tab::Holdings => "Holdings",
            Tab::Markets => "Markets",
            Tab::News => "News",
            Tab::Hand => "Hand",
            Tab::Rules => "Rules",
        }
    }

    /// The tabs on the bar, in order, which are also the keys 1 to 5:
    /// a sandbox's (docs/designs/sandbox.md, 4.1) or the family game's.
    pub fn bar(sandbox: bool) -> &'static [Tab] {
        if sandbox {
            &[Tab::Hand, Tab::Rules, Tab::Markets, Tab::News, Tab::Family]
        } else {
            &[Tab::Family, Tab::Holdings, Tab::Markets, Tab::News]
        }
    }
}

/// The accent of the interface: the hand's ink in a sandbox, the player's
/// family's colour in the family game, a warm brown in a recorded history.
pub fn accent(fam: Option<&FamilyView>, sandbox: bool) -> Color {
    if sandbox {
        return crate::sandbox::HAND_INK;
    }
    fam.and_then(|v| v.player()).map(|p| p.colour).unwrap_or(Color::srgb(0.55, 0.42, 0.30))
}

/// Which news the News panel lists.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NewsShow {
    Major,
    All,
    /// what the great families did (a live game only)
    Families,
}

#[derive(Resource)]
pub struct UiState {
    pub tab: Option<Tab>,
    pub news: NewsShow,
    /// the pointer is over a panel, so the camera and map clicks leave it alone
    pub hover: bool,
    /// bumped whenever panel content must be rebuilt for reasons other than the year
    pub dirty: u64,
    /// the family member whose posts are shown
    pub expanded: Option<usize>,
    /// the rival family opened in the list of great families
    pub rival: Option<usize>,
    /// how many items the News panel lists, newest first; Show more adds to it
    pub news_rows: usize,
    /// which half of the Markets panel shows
    pub markets: MarketsShow,
    /// the good whose history is open in the Markets panel
    pub good: Option<usize>,
    /// the Markets panel's charts and figures go month by month, not year by year
    pub by_month: bool,
    /// months that have come from a live game's simulation, each of which may change what a panel shows
    pub months: u64,
    built: Option<(Option<Tab>, usize, NewsShow, u64, usize)>,
    /// when the side panel was last drawn, in seconds: a panel that goes by the month is drawn a few times a second at most
    drawn_at: f32,
}

/// The News panel lists this many items, and as many again for each Show more.
const NEWS_ROWS: usize = 120;

impl Default for UiState {
    fn default() -> Self {
        UiState { tab: Some(Tab::News), news: NewsShow::Major, hover: false, dirty: 0, expanded: None, rival: None, news_rows: NEWS_ROWS, markets: MarketsShow::Prices, good: None, by_month: false, months: 0, built: None, drawn_at: 0.0 }
    }
}

/// The Markets panel has three halves: the goods and their prices, the island
/// as a whole, and its work and owners (work_charts.rs).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MarketsShow {
    Prices,
    Island,
    Work,
}

/// Whether a word of a `--view` says what the Markets panel shows: "island",
/// or "open:GOOD" for the goods with one good's history open; either may end
/// in ":months" for the panel month by month ("island:months", "open:coal:months").
pub fn markets_word(word: &str) -> bool {
    let w = word.to_ascii_lowercase();
    let w = w.strip_suffix(":months").unwrap_or(&w);
    w == "island" || w.starts_with("open:")
}

impl UiState {
    /// Set the Markets panel as a `markets_word` asks (for screenshots).
    pub fn show_markets(&mut self, word: &str, r: &Replay) {
        let w = word.to_ascii_lowercase();
        let (w, by_month) = match w.strip_suffix(":months") {
            Some(rest) => (rest, true),
            None => (w.as_str(), false),
        };
        let (show, good) = match w.strip_prefix("open:") {
            Some(g) => (MarketsShow::Prices, r.header.goods.iter().position(|x| x.name.eq_ignore_ascii_case(g) || x.label.eq_ignore_ascii_case(g))),
            None => (MarketsShow::Island, self.good),
        };
        if self.markets != show || self.good != good || self.by_month != by_month {
            self.markets = show;
            self.good = good;
            self.by_month = by_month;
            self.dirty += 1;
        }
    }
}

#[derive(Component)]
pub struct TabButton(pub Tab);
/// The word on a tab's button, beside its icon: dropped in a window too narrow for it (`tabs_fit`).
#[derive(Component)]
pub struct TabLabel;
/// The parts of the bar across the top, for `tabs_fit` to measure.
#[derive(Component, Clone, Copy, PartialEq)]
pub enum TopPart {
    /// the whole bar, from edge to edge
    Bar,
    /// the family's card
    Family,
    /// the card with the date, the speeds and the steps
    Clock,
    /// the tabs' card
    Tabs,
}
#[derive(Component, Clone, Copy, PartialEq)]
pub enum SpeedButton {
    Pause,
    Speed(f32),
}
#[derive(Component)]
pub struct DateText;
/// The small bar under the date: how far through its month the clock stands.
/// The simulation has no weeks or days, so it is a bar and not a number.
#[derive(Component)]
pub struct MonthBar;
/// The part of it that has gone by.
#[derive(Component)]
pub struct MonthBarFill;
/// One of the numbers under the family name.
#[derive(Component)]
pub struct FortuneSlot(pub usize);
#[derive(Component)]
pub struct FamilyNameText;
#[derive(Component)]
pub struct Crest;
#[derive(Component)]
pub struct CrestLetter;
#[derive(Component)]
pub struct SidePanel;
#[derive(Component)]
pub struct PanelTitle;
#[derive(Component)]
pub struct PanelBody;
/// Anything that should keep the pointer away from the map while hovered.
#[derive(Component)]
pub struct UiBlock;
/// Gets button colours on hover and press.
#[derive(Component)]
pub struct Hoverable;
#[derive(Component)]
pub struct NewsFilter;
/// Lists more of the news.
#[derive(Component)]
pub struct NewsMore;
#[derive(Component)]
pub struct TimelineTrack;
#[derive(Component)]
pub struct TimelineKnob;
#[derive(Component)]
pub struct HelpCard;
/// Holds the timeline's news ticks.
#[derive(Component)]
pub struct TickLayer;
/// The part of the timeline that has happened.
#[derive(Component)]
pub struct TimelineFill;
/// A pin in the hand's ink on the timeline at the year a sandbox's run on is
/// going to, shown while it goes.
#[derive(Component)]
pub struct RunOnMark;
/// Clicking this opens a tab (the family card opens the family).
#[derive(Component)]
pub struct OpenTab(pub Tab);
/// One month on, then stop.
#[derive(Component)]
pub struct NextMonthButton;
/// On to the year's turn, then stop.
#[derive(Component)]
pub struct YearEndButton;
/// The line under the date: the clock's pace, a year turning, when an order given now would happen.
#[derive(Component)]
pub struct StepNote;
/// By year or by month, in the Markets panel.
#[derive(Component)]
pub struct MonthsChip(pub bool);
/// A good's row in the Markets panel: click to open its history, or close it.
#[derive(Component)]
pub struct GoodRow(pub usize);
/// Choose a half of the Markets panel.
#[derive(Component)]
pub struct MarketsChip(pub MarketsShow);
/// A row that lights up under the pointer; `open` rows keep a ground of their own.
#[derive(Component)]
pub struct RowHover {
    pub open: bool,
}
/// A list that scrolls inside its card (`scroll_list`): the wheel turns it
/// while the pointer is over the card.
#[derive(Component)]
pub struct WheelList {
    pub card: Entity,
}
/// The thumb at the right edge of a list that scrolls, shown while the list
/// is longer than its room: how much of it shows, and where.
#[derive(Component)]
pub struct ScrollThumb(pub Entity);

pub fn plugin(app: &mut App) {
    app.init_resource::<UiState>().add_systems(PostStartup, setup_hud).add_systems(
        Update,
        (button_colours, row_colours, hotkeys, tab_clicks, speed_clicks, news_filter_click, news_more_click, markets_clicks, timeline_drag, active_styles, topbar_text, month_bar, tabs_fit, panel_visibility, panel_rebuild, wheel_lists, scroll_thumbs, timeline_knob, timeline_ticks).chain(),
    );
}

/// A list that scrolls inside the card `card`, which must carry a
/// `RelativeCursorPosition`: the wheel turns the list while the pointer is
/// over the card, and a thumb stands at its right edge while there is more
/// of it than shows, so that nothing a list holds is out of reach or out of
/// mind. `frame` is the room the list is given in the card, `list` the list's
/// own node (a column; it is made to scroll here) and `marker` what goes on
/// the list, which is the entity that takes the rows; `rows` puts in any
/// that are there from the start. The side panel and the note about what was
/// clicked are both made of this.
pub fn scroll_list<M: Bundle>(p: &mut ChildSpawnerCommands, card: Entity, frame: Node, list: Node, marker: M, rows: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn(Node { flex_direction: FlexDirection::Column, min_height: Val::Px(0.0), ..frame }).with_children(|fr| {
        let body = fr.spawn((Node { flex_direction: FlexDirection::Column, min_height: Val::Px(0.0), overflow: Overflow::scroll_y(), ..list }, ScrollPosition::default(), WheelList { card }, marker)).with_children(rows).id();
        fr.spawn((
            Node { position_type: PositionType::Absolute, right: Val::Px(0.0), top: Val::Px(0.0), width: Val::Px(THUMB_W), height: Val::Percent(100.0), border_radius: BorderRadius::all(Val::Px(THUMB_W / 2.0)), ..default() },
            BackgroundColor(THUMB),
            Visibility::Hidden,
            ScrollThumb(body),
        ));
    });
}

/// The thumb of a list that scrolls: a thin bar of the ink the rules are drawn in, a little darker.
const THUMB: Color = Color::srgb(0.80, 0.74, 0.64);
const THUMB_W: f32 = 3.0;
/// Room kept for it at the right of a list's rows.
pub const THUMB_ROOM: f32 = 7.0;

/// Where the thumb of a list stands: its top and its height as shares of the
/// list's room, or None when the whole list shows. `room` is the height the
/// list has, `content` how long its rows are and `at` how far it is scrolled.
pub fn thumb_span(room: f32, content: f32, at: f32) -> Option<(f32, f32)> {
    if !(room > 0.0) || content <= room + 0.5 {
        return None;
    }
    // never so short that it cannot be seen
    let h = (room / content).max(0.06);
    let top = (at / (content - room)).clamp(0.0, 1.0) * (1.0 - h);
    Some((top, h))
}

/// A key drawn as a little raised cap.
pub fn keycap(p: &mut ChildSpawnerCommands, f: &Handle<Font>, key: &str) {
    p.spawn((
        Node { padding: UiRect::axes(Val::Px(7.0), Val::Px(1.0)), border: UiRect::new(Val::Px(1.0), Val::Px(1.0), Val::Px(1.0), Val::Px(3.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() },
        BackgroundColor(Color::srgb(1.0, 0.99, 0.96)),
        BorderColor::all(Color::srgb(0.80, 0.74, 0.64)),
    ))
    .with_children(|k| {
        k.spawn(text(f, key, 13.0, INK));
    });
}

pub fn text(font: &Handle<Font>, s: &str, size: f32, color: Color) -> (Text, TextFont, TextColor) {
    (Text::new(s), TextFont { font: font.clone().into(), font_size: FontSize::Px(size), ..default() }, TextColor(color))
}

fn card_shadow() -> BoxShadow {
    BoxShadow::new(Color::srgba(0.25, 0.18, 0.10, 0.18), Val::Px(0.0), Val::Px(3.0), Val::Px(0.0), Val::Px(10.0))
}

fn button_node(px: f32, py: f32) -> Node {
    Node {
        padding: UiRect::axes(Val::Px(px), Val::Px(py)),
        border_radius: BorderRadius::all(Val::Px(10.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

// ------------------------------------------------------------------ building the interface

/// A floating paper card that keeps the pointer off the map.
pub fn card(node: Node) -> impl Bundle {
    (node, BackgroundColor(PAPER), card_shadow(), Interaction::default(), FocusPolicy::Block, UiBlock, GlobalZIndex(10))
}

/// Room for the date: "September 1784", the longest it gets.
const DATE_WIDTH: f32 = 166.0;

/// The gap between the cards across the top.
const TOP_GAP: f32 = 12.0;

/// The padding either side of the word on one of the clock's buttons: there
/// are eight of them in a game, so every pixel here is sixteen across the bar.
const CLOCK_BUTTON_PAD: f32 = 7.0;

/// Whether the tabs must drop their words for the three cards across the top
/// to fit: `bar` is the room from edge to edge, `family` and `clock` the
/// widths of those cards and `tabs` the tabs' card with its words, all in the
/// same pixels.
pub fn tabs_must_shrink(bar: f32, family: f32, clock: f32, tabs: f32) -> bool {
    family + clock + tabs + 2.0 * TOP_GAP > bar + 0.5
}

/// In a window too narrow for the three cards across the top, the tabs keep
/// their icons and drop their words, so that none of them is pushed off the
/// edge. The clock's card is the widest of the three (a month and a year, five
/// speeds and the two steps) and cannot give way: with the words kept, a
/// window 1280 wide would lose the Markets and News tabs. The words come back
/// when there is room for them again.
fn tabs_fit(parts: Query<(&ComputedNode, &TopPart)>, mut labels: Query<&mut Node, With<TabLabel>>, mut short: Local<bool>, mut with_words: Local<f32>) {
    let width = |part: TopPart| parts.iter().find(|(_, p)| **p == part).map_or(0.0, |(c, _)| c.size().x * c.inverse_scale_factor);
    // the tabs' width with their words is measured while they have them
    if !*short {
        *with_words = width(TopPart::Tabs);
    }
    let (bar, family, clock) = (width(TopPart::Bar), width(TopPart::Family), width(TopPart::Clock));
    // (before the first layout nothing has a width)
    if bar <= 0.0 || clock <= 0.0 || *with_words <= 0.0 {
        return;
    }
    let shrink = tabs_must_shrink(bar, family, clock, *with_words);
    if shrink != *short {
        *short = shrink;
        for mut n in &mut labels {
            n.display = if shrink { Display::None } else { Display::Flex };
        }
    }
}

pub fn setup_hud(mut commands: Commands, font: Res<UiFont>, data: Res<Data>, fam: Option<Res<FamilyView>>, icons: Res<crate::icons::Icons>, sandbox: Option<Res<crate::sandbox::Sandbox>>) {
    let f = &font.0;
    let sandbox = sandbox.is_some();
    // in a sandbox the hand's ink takes the family colour's place (docs/designs/sandbox.md, 4.1)
    let crest_col = if sandbox { crate::sandbox::HAND_INK } else { fam.as_ref().and_then(|v| v.player()).map(|p| p.colour).unwrap_or(Color::srgb(0.55, 0.47, 0.38)) };
    let family_name = fam.as_ref().and_then(|v| v.player()).map(|p| format!("The {} family", p.name)).unwrap_or_else(|| "A recorded history".into());
    // a sandbox's card is the island's: its number on the crest, its name beside it
    let seed = data.r.header.meta.seed;
    let card_name = if sandbox { island_name(seed) } else { family_name.clone() };

    // three cards across the top: the family, the date and the clock, the tabs
    commands
        .spawn((
            Node { position_type: PositionType::Absolute, left: Val::Px(14.0), right: Val::Px(14.0), top: Val::Px(12.0), justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Start, column_gap: Val::Px(TOP_GAP), ..default() },
            GlobalZIndex(10),
            TopPart::Bar,
        ))
        .with_children(|top| {
            // the side cards take what their contents need, the clock the rest, so a narrower window still fits
            top.spawn((Node { flex_shrink: 0.0, ..default() }, TopPart::Family)).with_children(|wrap| {
                wrap.spawn((
                    card(Node {
                        height: Val::Px(62.0),
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(12.0),
                        border_radius: BorderRadius::all(Val::Px(16.0)),
                        ..default()
                    }),
                    OpenTab(if sandbox { Tab::Hand } else { Tab::Family }),
                ))
                .with_children(|left| {
                    // a little shield with the family's initial, or in a sandbox the island's number
                    left.spawn((
                        Node {
                            width: Val::Px(32.0),
                            height: Val::Px(38.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(16.0), Val::Px(16.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(crest_col),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.75)),
                        Crest,
                    ))
                    .with_children(|c| {
                        if sandbox {
                            let number = seed.to_string();
                            c.spawn((text(f, &number, crest_number_size(&number), Color::WHITE), TextLayout::default().with_no_wrap(), CrestLetter));
                        } else {
                            let letter = family_name.trim_start_matches("The ").chars().next().unwrap_or('A').to_string();
                            c.spawn((text(f, &letter, 19.0, Color::WHITE), CrestLetter));
                        }
                    });
                    left.spawn(Node { flex_direction: FlexDirection::Column, ..default() }).with_children(|col| {
                        col.spawn((text(f, &card_name, 19.0, INK), FamilyNameText));
                        col.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(4.0), ..default() }).with_children(|row| {
                            let slot = |row: &mut ChildSpawnerCommands, i: usize, gap: bool| {
                                row.spawn((text(f, "", 13.5, MUTED), FortuneSlot(i), Node { margin: UiRect::right(Val::Px(if gap { 8.0 } else { 0.0 })), ..default() }));
                            };
                            if sandbox {
                                // the island's people and its works standing, for the year on the clock
                                row.spawn(crate::icons::icon(&icons.people, 15.0, MUTED));
                                slot(row, 0, true);
                                row.spawn(crate::icons::icon(&icons.works, 15.0, MUTED));
                                slot(row, 1, false);
                            } else if fam.is_some() {
                                slot(row, 0, true);
                                row.spawn(crate::icons::icon(&icons.coin, 15.0, Color::WHITE));
                                slot(row, 1, true);
                                row.spawn(crate::icons::icon(&icons.works, 15.0, MUTED));
                                slot(row, 2, true);
                                row.spawn(crate::icons::icon(&icons.land, 15.0, MUTED));
                                slot(row, 3, false);
                            } else {
                                row.spawn(crate::icons::icon(&icons.people, 15.0, MUTED));
                                slot(row, 0, true);
                                slot(row, 1, false);
                            }
                        });
                    });
                });
            });
            top.spawn(Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(4.0), ..default() }).with_children(|wrap| {
                wrap.spawn((
                    card(Node {
                        height: Val::Px(62.0),
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(14.0),
                        border_radius: BorderRadius::all(Val::Px(16.0)),
                        ..default()
                    }),
                    TopPart::Clock,
                ))
                .with_children(|mid| {
                    // as wide as the longest month and the note under it, so the buttons do not shift as the months go by
                    mid.spawn(Node { flex_direction: FlexDirection::Column, width: Val::Px(DATE_WIDTH), ..default() }).with_children(|col| {
                        col.spawn((text(f, "", 21.0, INK), TextLayout::default().with_no_wrap(), DateText));
                        // how far through the month the clock stands
                        col.spawn((Node { width: Val::Px(DATE_WIDTH - 8.0), height: Val::Px(3.0), margin: UiRect::vertical(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(2.0)), ..default() }, BackgroundColor(RULE), MonthBar))
                            .with_children(|bar| {
                                bar.spawn((Node { width: Val::Percent(0.0), height: Val::Percent(100.0), border_radius: BorderRadius::all(Val::Px(2.0)), ..default() }, BackgroundColor(crest_col), MonthBarFill));
                            });
                        col.spawn((text(f, "", 11.5, MUTED), TextLayout::default().with_no_wrap(), StepNote));
                    });
                    mid.spawn(Node { column_gap: Val::Px(4.0), ..default() }).with_children(|row| {
                        for (label, b) in std::iter::once(("Pause", SpeedButton::Pause)).chain(clock::SPEEDS.iter().map(|s| (s.name, SpeedButton::Speed(s.rate)))) {
                            row.spawn((button_node(CLOCK_BUTTON_PAD, 5.0), BackgroundColor(BTN), Interaction::default(), FocusPolicy::Block, Hoverable, b)).with_children(|bt| {
                                bt.spawn((text(f, label, 14.0, INK), TextLayout::default().with_no_wrap()));
                            });
                        }
                        if data.live {
                            // a month on, or on to the year's turn, then stop: the two rhythms of a turn
                            row.spawn((Node { margin: UiRect::left(Val::Px(6.0)), ..button_node(CLOCK_BUTTON_PAD, 5.0) }, BackgroundColor(BTN), Interaction::default(), FocusPolicy::Block, Hoverable, NextMonthButton)).with_children(|bt| {
                                bt.spawn((text(f, "Next month", 14.0, INK), TextLayout::default().with_no_wrap()));
                            });
                            row.spawn((button_node(CLOCK_BUTTON_PAD, 5.0), BackgroundColor(BTN), Interaction::default(), FocusPolicy::Block, Hoverable, YearEndButton)).with_children(|bt| {
                                bt.spawn((text(f, "To January", 14.0, INK), TextLayout::default().with_no_wrap()));
                            });
                        }
                    });
                });
                // decisions waiting: a tab hanging under the clock, shown only while there are some
                if data.live {
                    wrap.spawn((
                        Node { display: Display::None, padding: UiRect::axes(Val::Px(14.0), Val::Px(5.0)), border_radius: BorderRadius::all(Val::Px(12.0)), ..default() },
                        BackgroundColor(crest_col),
                        card_shadow(),
                        Interaction::default(),
                        FocusPolicy::Block,
                        UiBlock,
                        crate::game::DecisionBadge,
                    ))
                    .with_children(|bt| {
                        bt.spawn((text(f, "", 14.5, Color::WHITE), TextLayout::default().with_no_wrap()));
                    });
                }
            });
            top.spawn((Node { flex_shrink: 0.0, justify_content: JustifyContent::End, ..default() }, TopPart::Tabs)).with_children(|wrap| {
                wrap.spawn(card(Node {
                    height: Val::Px(62.0),
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    border_radius: BorderRadius::all(Val::Px(16.0)),
                    ..default()
                }))
                .with_children(|right| {
                    for &t in Tab::bar(sandbox) {
                        let ic = match t {
                            Tab::Family => &icons.family,
                            Tab::Holdings => &icons.holdings,
                            Tab::Markets => &icons.markets,
                            Tab::News => &icons.news,
                            Tab::Hand => &icons.pen,
                            Tab::Rules => &icons.scales,
                        };
                        right.spawn((Node { column_gap: Val::Px(6.0), ..button_node(11.0, 7.0) }, BackgroundColor(BTN), Interaction::default(), FocusPolicy::Block, Hoverable, TabButton(t))).with_children(|bt| {
                            bt.spawn(crate::icons::icon(ic, 16.0, INK));
                            bt.spawn((text(f, t.label(sandbox), 15.0, INK), TabLabel));
                        });
                    }
                });
            });
        });

    // the side panel
    commands
        .spawn((
            card(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(14.0),
                top: Val::Px(86.0),
                bottom: Val::Px(70.0),
                width: Val::Px(380.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::new(Val::Px(16.0), Val::Px(10.0), Val::Px(12.0), Val::Px(12.0)),
                border_radius: BorderRadius::all(Val::Px(16.0)),
                ..default()
            }),
            RelativeCursorPosition::default(),
            SidePanel,
        ))
        .with_children(|p| {
            let card = p.target_entity();
            p.spawn((text(f, "News", 22.0, INK), PanelTitle, Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() }));
            scroll_list(p, card, Node { flex_grow: 1.0, ..default() }, Node { flex_grow: 1.0, row_gap: Val::Px(6.0), padding: UiRect::right(Val::Px(THUMB_ROOM)), ..default() }, PanelBody, |_| {});
        });

    // a reminder of the keys, under the panel
    commands
        .spawn(card(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(14.0),
            bottom: Val::Px(12.0),
            width: Val::Px(380.0),
            height: Val::Px(46.0),
            padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
            align_items: AlignItems::Center,
            column_gap: Val::Px(14.0),
            border_radius: BorderRadius::all(Val::Px(14.0)),
            ..default()
        }))
        .with_children(|h| {
            for (key, what) in [("H", "help"), ("Space", "play"), ("Drag", "look around")] {
                h.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(5.0), ..default() }).with_children(|kv| {
                    keycap(kv, f, key);
                    kv.spawn(text(f, what, 13.5, MUTED));
                });
            }
        });

    // the help card, shown with H
    commands
        .spawn((
            Node { position_type: PositionType::Absolute, left: Val::Px(0.0), right: Val::Px(0.0), top: Val::Px(0.0), bottom: Val::Px(0.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            GlobalZIndex(30),
            // ISOSTATE_HELP=1 opens it from the start, to see it in a screenshot
            if std::env::var_os("ISOSTATE_HELP").is_some() { Visibility::Inherited } else { Visibility::Hidden },
            HelpCard,
        ))
        .with_children(|wrap| {
            // (a card is on layer 10 of its own, which would put it under the sandbox's tray and the
            // toasts beside it; the help card is above them all)
            wrap.spawn(card(Node { flex_direction: FlexDirection::Column, padding: UiRect::axes(Val::Px(26.0), Val::Px(20.0)), row_gap: Val::Px(7.0), border_radius: BorderRadius::all(Val::Px(18.0)), ..default() })).insert(GlobalZIndex(30)).with_children(|c| {
                c.spawn((text(f, "How to look around", 22.0, INK), Node { margin: UiRect::bottom(Val::Px(6.0)), ..default() }));
                if sandbox {
                    // what the tray is, in a few lines, before the keys
                    for line in [
                        "The tray under the clock holds the hand's tools. Take one (Money, People, Harvest),",
                        "choose from its chips, then click the map: a town, or anywhere in its district.",
                        "Escape puts the tool down. Run on plays 1, 10 or 25 years and stops in January.",
                        "Take back the last edit, beside Run on, puts the island back as it stood before it.",
                    ] {
                        c.spawn(text(f, line, 14.5, MUTED));
                    }
                    c.spawn(Node { height: Val::Px(4.0), ..default() });
                }
                let keys: &[(&str, &str)] = &[
                    ("Drag", "move across the island"),
                    ("Right-drag", "turn and tilt the view"),
                    ("Scroll", "come closer or stand back"),
                    ("W A S D", "walk the view, Q and E to turn"),
                    ("Click", "ask about a works, a town or a square"),
                    ("Space", "start or stop the clock"),
                    ("Right", "one month on, then stop"),
                    ("Page Up", "on to the year's turn, in January"),
                    ("Left", "back a year (Page Down: ten)"),
                    ("Up, Down", "faster or slower"),
                ];
                let tabs: (&str, &str) = if sandbox { ("1 2 3 4 5", "hand, rules, markets, news, families") } else { ("1 2 3 4", "family, holdings, markets, news") };
                let more: &[(&str, &str)] = &[
                    // (task A) two more: when town ground joined its town, and who lives on it
                    ("M", "map views: rent, markets, reach, prices, wages, town growth, residents"),
                    ("G", "the next good, when prices are mapped"),
                    ("T", "miniature focus on or off"),
                    // note.rs: her words, kept where she stands
                    ("N", "a note in your words, kept with a picture of the screen"),
                    ("Esc", if sandbox { "put the tool down, or close the note about what you clicked" } else { "close the note about what you clicked" }),
                ];
                let f3: &[(&str, &str)] = if sandbox { &[("F3", "how long each frame takes to draw, as a strip")] } else { &[] };
                for &(key, what) in keys.iter().chain(std::iter::once(&tabs)).chain(more).chain(f3) {
                    c.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                        row.spawn(Node { width: Val::Px(120.0), justify_content: JustifyContent::End, ..default() }).with_children(|kc| keycap(kc, f, key));
                        row.spawn(text(f, what, 15.0, INK));
                    });
                }
                // her question of 19 November: the backlog, read from docs/BACKLOG.md, and her notes of the day
                c.spawn(Node { margin: UiRect::top(Val::Px(8.0)), ..default() }).with_children(|row| small_button(row, f, "The backlog and your notes of today", crate::backlog_seen::OpenBacklog));
                c.spawn((text(f, "Press H again to close this.", 13.0, MUTED), Node { margin: UiRect::top(Val::Px(8.0)), ..default() }));
            });
        });

    // the timeline
    let r = &data.r;
    commands
        .spawn(card(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(408.0),
            right: Val::Px(14.0),
            bottom: Val::Px(12.0),
            height: Val::Px(46.0),
            padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
            align_items: AlignItems::Center,
            column_gap: Val::Px(12.0),
            border_radius: BorderRadius::all(Val::Px(14.0)),
            ..default()
        }))
        .with_children(|tlb| {
            tlb.spawn(text(f, &format!("{}", r.y0), 13.0, MUTED));
            tlb.spawn((
                Node { flex_grow: 1.0, height: Val::Px(26.0), ..default() },
                Interaction::default(),
                RelativeCursorPosition::default(),
                TimelineTrack,
            ))
            .with_children(|track| {
                // the rail
                track.spawn((
                    Node { position_type: PositionType::Absolute, left: Val::Px(0.0), right: Val::Px(0.0), top: Val::Px(17.0), height: Val::Px(3.0), border_radius: BorderRadius::all(Val::Px(2.0)), ..default() },
                    BackgroundColor(RULE),
                ));
                // the years there are so far (all of them in a replay)
                track.spawn((
                    Node { position_type: PositionType::Absolute, left: Val::Px(0.0), width: Val::Percent(100.0), top: Val::Px(17.0), height: Val::Px(3.0), border_radius: BorderRadius::all(Val::Px(2.0)), ..default() },
                    // (in a sandbox the hand's ink, paler than the knob)
                    BackgroundColor(if sandbox { crate::sandbox::HAND_INK.with_alpha(0.45) } else { Color::srgb(0.80, 0.74, 0.64) }),
                    TimelineFill,
                ));
                // a tick for each major piece of news, filled in by timeline_ticks
                track.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(0.0), right: Val::Px(0.0), top: Val::Px(0.0), bottom: Val::Px(0.0), ..default() }, TickLayer));
                // (a sandbox) the year a run on is going to: a thin stem through the rail with a
                // round head, in the hand's ink, hidden until a run on is under way
                if sandbox {
                    let ink = crate::sandbox::HAND_INK;
                    track
                        .spawn((Node { position_type: PositionType::Absolute, left: Val::Percent(0.0), top: Val::Px(0.0), width: Val::Px(9.0), height: Val::Px(26.0), margin: UiRect::left(Val::Px(-4.5)), display: Display::None, ..default() }, RunOnMark))
                        .with_children(|pin| {
                            pin.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(3.5), top: Val::Px(6.0), width: Val::Px(2.0), height: Val::Px(19.0), border_radius: BorderRadius::all(Val::Px(1.0)), ..default() }, BackgroundColor(ink)));
                            pin.spawn((
                                Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Px(9.0), height: Val::Px(9.0), border: UiRect::all(Val::Px(2.0)), border_radius: BorderRadius::all(Val::Px(4.5)), ..default() },
                                BackgroundColor(Color::WHITE),
                                BorderColor::all(ink),
                            ));
                        });
                }
                // the knob with the year above it
                track.spawn((
                    Node { position_type: PositionType::Absolute, left: Val::Percent(100.0), top: Val::Px(11.0), width: Val::Px(14.0), height: Val::Px(14.0), margin: UiRect::left(Val::Px(-7.0)), border_radius: BorderRadius::all(Val::Px(7.0)), border: UiRect::all(Val::Px(2.0)), ..default() },
                    BackgroundColor(crest_col),
                    BorderColor::all(Color::WHITE),
                    BoxShadow::new(Color::srgba(0.25, 0.18, 0.10, 0.25), Val::Px(0.0), Val::Px(1.0), Val::Px(0.0), Val::Px(3.0)),
                    TimelineKnob,
                ));
            });
            tlb.spawn(text(f, &format!("{}", r.y0 + r.ny as i32 - 1), 13.0, MUTED));
        });
}

// ------------------------------------------------------------------ behaviour

/// Runs before the camera and the map picking, so neither acts on a click meant for a panel.
pub fn ui_hover(mut ui: ResMut<UiState>, q: Query<&Interaction>) {
    ui.hover = q.iter().any(|i| *i != Interaction::None);
}

fn toggle_tab(ui: &mut UiState, t: Tab) {
    ui.tab = if ui.tab == Some(t) { None } else { Some(t) };
}

fn hotkeys(keys: Res<ButtonInput<KeyCode>>, mut ui: ResMut<UiState>, mut help: Query<&mut Visibility, With<HelpCard>>, cards: Res<crate::game::CardOpen>, sandbox: Option<Res<crate::sandbox::Sandbox>>) {
    // the panels and the help wait while a card wants an answer
    if cards.0 {
        return;
    }
    // the keys 1 to 5 are the tabs on the bar, in its order
    let digits = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5];
    for (&k, &t) in digits.iter().zip(Tab::bar(sandbox.is_some())) {
        if keys.just_pressed(k) {
            toggle_tab(&mut ui, t);
        }
    }
    if keys.just_pressed(KeyCode::KeyH) {
        for mut v in &mut help {
            *v = if *v == Visibility::Hidden { Visibility::Inherited } else { Visibility::Hidden };
        }
    }
}

/// The side panel shows while a tab is chosen.
fn panel_visibility(ui: Res<UiState>, mut panel: Query<&mut Visibility, With<SidePanel>>) {
    let want = if ui.tab.is_some() { Visibility::Inherited } else { Visibility::Hidden };
    for mut v in &mut panel {
        v.set_if_neq(want);
    }
}

/// Rows that can be clicked show it under the pointer, and are otherwise bare.
fn row_colours(mut q: Query<(&Interaction, &RowHover, &mut BackgroundColor), Changed<Interaction>>) {
    for (i, row, mut bg) in &mut q {
        bg.0 = match i {
            Interaction::Pressed => BTN_HOVER,
            Interaction::Hovered => BTN,
            Interaction::None if row.open => PAPER_2,
            Interaction::None => Color::NONE,
        };
    }
}

fn markets_clicks(
    mut ui: ResMut<UiState>,
    mut ov: ResMut<crate::overlay::MapOverlay>,
    rows: Query<(&Interaction, &GoodRow), Changed<Interaction>>,
    chips: Query<(&Interaction, &MarketsChip), Changed<Interaction>>,
    months: Query<(&Interaction, &MonthsChip), Changed<Interaction>>,
) {
    for (i, c) in &months {
        if *i == Interaction::Pressed && ui.by_month != c.0 {
            ui.by_month = c.0;
            ui.dirty += 1;
        }
    }
    for (i, g) in &rows {
        if *i == Interaction::Pressed {
            ui.good = if ui.good == Some(g.0) { None } else { Some(g.0) };
            ui.dirty += 1;
            // the price map follows: the good whose history is open is the good it shows
            if ui.good.is_some() && ov.good != g.0 {
                ov.good = g.0;
            }
        }
    }
    for (i, c) in &chips {
        if *i == Interaction::Pressed && ui.markets != c.0 {
            ui.markets = c.0;
            ui.dirty += 1;
        }
    }
}

fn button_colours(mut q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Hoverable>)>) {
    for (i, mut bg) in &mut q {
        bg.0 = match i {
            Interaction::Pressed => BTN_DOWN,
            Interaction::Hovered => BTN_HOVER,
            Interaction::None => BTN,
        };
    }
}

/// Selected tabs and the current speed are shown in the family's colour (the hand's ink in a sandbox).
#[allow(clippy::too_many_arguments)]
fn active_styles(
    ui: Res<UiState>,
    tl: Res<Timeline>,
    fam: Option<Res<FamilyView>>,
    sandbox: Option<Res<crate::sandbox::Sandbox>>,
    tabs: Query<(&TabButton, &Interaction, &Children, Entity), Without<SpeedButton>>,
    speeds: Query<(&SpeedButton, &Interaction, &Children, Entity), Without<TabButton>>,
    mut bgs: Query<&mut BackgroundColor>,
    mut cols: Query<&mut TextColor>,
    mut imgs: Query<&mut ImageNode>,
) {
    let accent = accent(fam.as_deref(), sandbox.is_some());
    let mut paint = |e: Entity, children: &Children, on: bool, inter: &Interaction| {
        if let Ok(mut bg) = bgs.get_mut(e) {
            bg.0 = if on { accent } else { match inter { Interaction::Pressed => BTN_DOWN, Interaction::Hovered => BTN_HOVER, Interaction::None => BTN } };
        }
        for c in children.iter() {
            if let Ok(mut tc) = cols.get_mut(c) {
                tc.0 = if on { Color::WHITE } else { INK };
            }
            if let Ok(mut im) = imgs.get_mut(c) {
                im.color = if on { Color::WHITE } else { INK };
            }
        }
    };
    for (t, i, ch, e) in &tabs {
        paint(e, ch, ui.tab == Some(t.0), i);
    }
    for (s, i, ch, e) in &speeds {
        let on = match s {
            SpeedButton::Pause => !tl.playing,
            SpeedButton::Speed(v) => tl.playing && (tl.speed - v).abs() < 1e-3,
        };
        paint(e, ch, on, i);
    }
}

fn tab_clicks(mut ui: ResMut<UiState>, q: Query<(&Interaction, &TabButton), Changed<Interaction>>, open: Query<(&Interaction, &OpenTab), Changed<Interaction>>) {
    for (i, t) in &q {
        if *i == Interaction::Pressed {
            toggle_tab(&mut ui, t.0);
        }
    }
    for (i, t) in &open {
        if *i == Interaction::Pressed {
            ui.tab = Some(t.0);
        }
    }
}

fn speed_clicks(mut tl: ResMut<Timeline>, data: Res<Data>, q: Query<(&Interaction, &SpeedButton), Changed<Interaction>>) {
    for (i, s) in &q {
        if *i != Interaction::Pressed {
            continue;
        }
        match s {
            SpeedButton::Pause => tl.playing = false,
            SpeedButton::Speed(v) => {
                if !tl.playing && tl.t >= (data.r.ny - 1) as f32 && !data.live {
                    tl.t = 0.0;
                }
                tl.playing = true;
                tl.speed = *v;
            }
        }
    }
}

fn news_filter_click(mut ui: ResMut<UiState>, data: Res<Data>, q: Query<&Interaction, (Changed<Interaction>, With<NewsFilter>)>) {
    for i in &q {
        if *i == Interaction::Pressed {
            ui.news = match ui.news {
                NewsShow::Major => NewsShow::All,
                NewsShow::All if data.live => NewsShow::Families,
                _ => NewsShow::Major,
            };
            ui.news_rows = NEWS_ROWS;
        }
    }
}

fn news_more_click(mut ui: ResMut<UiState>, q: Query<&Interaction, (Changed<Interaction>, With<NewsMore>)>) {
    for i in &q {
        if *i == Interaction::Pressed {
            ui.news_rows += NEWS_ROWS;
            ui.dirty += 1;
        }
    }
}

/// The bar under the date fills as the month goes by; it is hidden where the
/// date is a year and no month.
fn month_bar(
    tl: Res<Timeline>,
    data: Res<Data>,
    live: Option<Res<crate::live::Live>>,
    stills: Option<Res<crate::ShotPlanArgs>>,
    mut bar: Query<&mut Visibility, With<MonthBar>>,
    mut fill: Query<&mut Node, With<MonthBarFill>>,
) {
    let date = crate::shown_date(&tl, &data, live.as_deref(), stills.is_some());
    for mut v in &mut bar {
        v.set_if_neq(if date.is_some() { Visibility::Inherited } else { Visibility::Hidden });
    }
    let w = Val::Percent(date.map_or(0.0, |d| (d.part * 100.0).round()));
    for mut n in &mut fill {
        if n.width != w {
            n.width = w;
        }
    }
}

/// The year's end a press on the timeline stands for, from where the pointer
/// is across the track (`RelativeCursorPosition::normalized`, -0.5 to 0.5).
pub fn timeline_year(x: f32, last: usize) -> usize {
    ((x + 0.5).clamp(0.0, 1.0) * last as f32).round() as usize
}

/// A press on the timeline puts the clock at the year pressed. In a sandbox a
/// press ahead of the present runs on to the January of that year
/// (`Sandbox::run_to`), and one while a run on is under way moves its year;
/// elsewhere the clock goes no further than the present.
fn timeline_drag(
    mut tl: ResMut<Timeline>,
    data: Res<Data>,
    q: Query<(&Interaction, &RelativeCursorPosition), With<TimelineTrack>>,
    mut sandbox: Option<ResMut<crate::sandbox::Sandbox>>,
    live: Option<Res<crate::live::Live>>,
    time: Res<Time>,
) {
    for (i, rel) in &q {
        if *i != Interaction::Pressed {
            continue;
        }
        if let Some(n) = rel.normalized {
            let k = timeline_year(n.x, data.r.ny.max(1) - 1);
            if let (Some(sb), Some(live)) = (sandbox.as_mut(), live.as_ref()) {
                if k > data.latest() {
                    sb.run_to(&mut tl, crate::sandbox::Present::of(&data, live), k, None, time.elapsed_secs());
                    continue;
                }
            }
            tl.playing = false;
            tl.t = (k as f32).min(data.latest() as f32);
        }
    }
}

fn timeline_knob(
    tl: Res<Timeline>,
    data: Res<Data>,
    sandbox: Option<Res<crate::sandbox::Sandbox>>,
    mut q: Query<&mut Node, (With<TimelineKnob>, Without<TimelineFill>, Without<RunOnMark>)>,
    mut fill: Query<&mut Node, (With<TimelineFill>, Without<TimelineKnob>, Without<RunOnMark>)>,
    mut mark: Query<&mut Node, (With<RunOnMark>, Without<TimelineKnob>, Without<TimelineFill>)>,
) {
    let last = (data.r.ny.max(2) - 1) as f32;
    for mut n in &mut q {
        n.left = Val::Percent((tl.t / last * 100.0).clamp(0.0, 100.0));
    }
    let w = Val::Percent((data.latest() as f32 / last * 100.0).clamp(0.0, 100.0));
    for mut n in &mut fill {
        if n.width != w {
            n.width = w;
        }
    }
    // the year a run on is going to, while it goes
    let to = sandbox.and_then(|s| s.target());
    for mut n in &mut mark {
        let (display, left) = match to {
            Some(k) => (Display::Flex, Val::Percent((k as f32 / last * 100.0).clamp(0.0, 100.0))),
            None => (Display::None, n.left),
        };
        if n.display != display || n.left != left {
            n.display = display;
            n.left = left;
        }
    }
}

/// How tall the tallest tick of the news stands on the timeline, in pixels.
const NEWS_TICK_TALLEST: f32 = 16.0;

/// A tick on the timeline for a thing done by hand.
#[derive(Component)]
pub struct HandTick;

/// How many ticks of the hand the timeline holds, for a script's `print`.
pub fn hand_ticks(world: &mut World) -> usize {
    world.query_filtered::<(), With<HandTick>>().iter(world).count()
}

/// News ticks along the timeline, redrawn as a live game adds news. In a
/// sandbox each thing done by hand has a tick of its own in the hand's ink,
/// half as tall again as the tallest of the news and drawn over them.
fn timeline_ticks(mut commands: Commands, data: Res<Data>, sandbox: Option<Res<crate::sandbox::Sandbox>>, layer: Query<(Entity, Option<&Children>), With<TickLayer>>, mut drawn: Local<Option<(usize, usize)>>) {
    let _frames = crate::frames::span(crate::frames::Sys::Hud);
    let r = &data.r;
    // chronicle: the marks of news already drawn can change, so the count of major items is part of the key
    let n = (r.header.events.len(), r.header.events.iter().filter(|e| e.major()).count());
    if *drawn == Some(n) {
        return;
    }
    let Ok((e, children)) = layer.single() else { return };
    *drawn = Some(n);
    if let Some(ch) = children {
        for c in ch.iter() {
            commands.entity(c).despawn();
        }
    }
    let span = (r.ny.max(2) - 1) as f32;
    // where the knob stands when the clock is on the item's date (`clock::news_time`)
    let at = |year: f32| ((year - r.y0 as f32 - 1.0) / span * 100.0).clamp(0.0, 100.0);
    // (a recording or a family game has no hand: what it has of kind "hand" is news like any other)
    let by_hand = |e: &&crate::replay::Event| sandbox.is_some() && e.kind == "hand";
    commands.entity(e).with_children(|track| {
        for e in r.header.events.iter().filter(|e| e.major() && !by_hand(e)) {
            let h = 6.0 + (NEWS_TICK_TALLEST - 6.0) * ((e.weight - 0.6) / 0.4).clamp(0.0, 1.0);
            track.spawn((
                Node { position_type: PositionType::Absolute, left: Val::Percent(at(e.year)), top: Val::Px(16.0 - h), width: Val::Px(2.0), height: Val::Px(h), border_radius: BorderRadius::all(Val::Px(1.0)), ..default() },
                BackgroundColor(kind_colour(&e.kind).with_alpha(0.8)),
            ));
        }
        // the hand's, after the news so that they stand over it
        let h = NEWS_TICK_TALLEST * 1.5;
        for e in r.header.events.iter().filter(by_hand) {
            track.spawn((
                HandTick,
                Node { position_type: PositionType::Absolute, left: Val::Percent(at(e.year)), top: Val::Px(16.0 - h), width: Val::Px(3.0), height: Val::Px(h), margin: UiRect::left(Val::Px(-0.5)), border_radius: BorderRadius::all(Val::Px(1.5)), ..default() },
                BackgroundColor(crate::sandbox::HAND_INK),
            ));
        }
    });
}

fn season_name(ph: f32) -> &'static str {
    match ((ph * 4.0 + 0.5).floor() as i32).rem_euclid(4) {
        0 => "Winter",
        1 => "Spring",
        2 => "Summer",
        _ => "Autumn",
    }
}

#[allow(clippy::too_many_arguments)]
fn topbar_text(
    tl: Res<Timeline>,
    data: Res<Data>,
    sm: Option<Res<crate::SeasonMats>>,
    clock: Res<crate::SeasonClock>,
    fam: Option<Res<FamilyView>>,
    live: Option<Res<crate::live::Live>>,
    stills: Option<Res<crate::ShotPlanArgs>>,
    sandbox: Option<Res<crate::sandbox::Sandbox>>,
    mut date: Query<&mut Text, (With<DateText>, Without<FortuneSlot>, Without<FamilyNameText>)>,
    mut slots: Query<(&mut Text, &FortuneSlot), (Without<DateText>, Without<FamilyNameText>)>,
    mut name: Query<&mut Text, (With<FamilyNameText>, Without<DateText>, Without<FortuneSlot>)>,
) {
    let r = &data.r;
    let k = (tl.t.floor() as usize).min(r.ny - 1);
    let year = r.y0 + k as i32;
    // the month and the year, where the clock stands in a month; the year alone
    // where it stands on a year gone by (and in every still). A still of a
    // chosen season names the season, which is the phase the ground, the trees
    // and the sun are showing.
    let forced = sm.as_ref().and_then(|m| m.forced);
    let s = match (forced, crate::shown_date(&tl, &data, live.as_deref(), stills.is_some())) {
        (Some(_), _) => format!("{} {}", season_name(crate::season_phase(&clock, forced)), year),
        (None, Some(d)) => clock::month_year(d),
        (None, None) => format!("{year}"),
    };
    for mut t in &mut date {
        if t.0 != s {
            t.0 = s.clone();
        }
    }
    let vals: Vec<String> = match fam.as_ref().and_then(|v| v.player()) {
        // the island's card: its name stays, its figures are those of the year on the clock
        _ if sandbox.is_some() => island_figures(r, k).to_vec(),
        Some(p) => {
            // the family is known as it stands today: beside an earlier date
            // its card says which year its figures are of (in the name, the
            // shorter line, so the cards beside it stay where they are)
            let past = live.as_ref().is_some_and(|l| crate::game::now_of(k, data.latest(), l.seen) == crate::game::Now::Past);
            let who = if past { format!("The {} family in {}", p.name, r.y0 + data.latest() as i32) } else { format!("The {} family", p.name) };
            for mut t in &mut name {
                if t.0 != who {
                    t.0 = who.clone();
                }
            }
            vec![format!("Worth {}", money(p.net_worth)), money(p.cash), format!("{}", p.works), format!("{}", p.land)]
        }
        None => {
            let pop = replay::annual(r, "pop", k).unwrap_or(0.0);
            vec![crate::fmt_pop(pop), format!("replay of seed {}", r.header.meta.seed)]
        }
    };
    for (mut t, slot) in &mut slots {
        if let Some(v) = vals.get(slot.0) {
            if t.0 != *v {
                t.0 = v.clone();
            }
        }
    }
}

/// The name on a sandbox's card: "Island 7".
pub fn island_name(seed: u64) -> String {
    format!("Island {seed}")
}

/// How large the island's number is written on the crest, which is 32 pixels
/// across: one or two figures as large as a family's initial, more smaller.
fn crest_number_size(number: &str) -> f32 {
    match number.chars().count() {
        0..=2 => 19.0,
        3 => 14.0,
        4 => 11.0,
        _ => 8.5,
    }
}

/// The figures on the island's card for year `k`, from the record: its
/// people, and its works standing (the record's yearly count of works).
pub fn island_figures(r: &Replay, k: usize) -> [String; 2] {
    let people = replay::annual(r, "pop", k).unwrap_or(0.0);
    let works = replay::annual(r, "firms", k).unwrap_or(0.0);
    // bare numbers beside their icons, as the family's card has them
    [crate::fmt_pop(people), thousands(works.round())]
}

/// For sorting the biggest first. A number that is not a number goes last
/// rather than stopping the game.
pub fn biggest_first(a: f32, b: f32) -> std::cmp::Ordering {
    let key = |v: f32| if v.is_nan() { f32::NEG_INFINITY } else { v };
    key(b).total_cmp(&key(a))
}

pub fn money(v: f32) -> String {
    let a = v.abs();
    // switch units where rounding would carry: 999,600 is £1.00m, not £1000k
    let s = if a >= 999_500.0 { format!("\u{00a3}{:.2}m", a / 1e6) } else if a >= 9_999.5 { format!("\u{00a3}{:.0}k", a / 1e3) } else { format!("\u{00a3}{:.0}", a) };
    if v < 0.0 { format!("-{s}") } else { s }
}

/// Where a list is scrolled to after the wheel turns `dy` pixels (up is
/// positive). `asked` is where it was last asked to be and `shown` where the
/// layout could put it, which is never past the end: starting from the lesser,
/// wheeling down at the foot of the list does not pile up a distance that
/// would have to be wheeled back before the list moved again.
fn scroll_to(asked: f32, shown: f32, dy: f32) -> f32 {
    (asked.min(shown) - dy).max(0.0)
}

/// Turn the wheel over the side panel.
pub fn scroll_panel(s: &mut ScrollPosition, c: &ComputedNode, dy: f32) {
    s.0.y = scroll_to(s.0.y, c.scroll_position.y * c.inverse_scale_factor, dy);
}

/// The wheel turns a list while the pointer is over its card and the card shows.
fn wheel_lists(scroll: Res<AccumulatedMouseScroll>, cards: Query<(&RelativeCursorPosition, &InheritedVisibility)>, mut lists: Query<(&WheelList, &mut ScrollPosition, &ComputedNode)>) {
    let dy = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y * 36.0,
        MouseScrollUnit::Pixel => scroll.delta.y,
    };
    if dy == 0.0 {
        return;
    }
    for (list, mut s, c) in &mut lists {
        if cards.get(list.card).is_ok_and(|(over, shown)| over.cursor_over() && shown.get()) {
            scroll_panel(&mut s, c, dy);
        }
    }
}

/// Each list's thumb stands where the list is scrolled to, and shows only while there is more of the list than its room.
fn scroll_thumbs(lists: Query<&ComputedNode, With<WheelList>>, mut thumbs: Query<(&ScrollThumb, &mut Node, &mut Visibility)>) {
    for (thumb, mut node, mut vis) in &mut thumbs {
        let span = lists.get(thumb.0).ok().and_then(|c| thumb_span(c.size().y, c.content_size().y, c.scroll_position.y));
        vis.set_if_neq(if span.is_some() { Visibility::Inherited } else { Visibility::Hidden });
        if let Some((top, h)) = span {
            let (top, h) = (Val::Percent(top * 100.0), Val::Percent(h * 100.0));
            if node.top != top || node.height != h {
                (node.top, node.height) = (top, h);
            }
        }
    }
}

// ------------------------------------------------------------------ panel content

#[allow(clippy::too_many_arguments)]
fn panel_rebuild(
    mut commands: Commands,
    mut ui: ResMut<UiState>,
    tl: Res<Timeline>,
    data: Res<Data>,
    font: Res<UiFont>,
    fam: Option<Res<FamilyView>>,
    live: Option<Res<crate::live::Live>>,
    mut images: ResMut<Assets<Image>>,
    mut body: Query<(Entity, Option<&Children>, &mut ScrollPosition), With<PanelBody>>,
    mut title: Query<&mut Text, With<PanelTitle>>,
    mut last_list: Local<Option<(Tab, NewsShow)>>,
    time: Res<Time>,
    sandbox: Option<Res<crate::sandbox::Sandbox>>,
    // what the Hand and Rules tabs read, in their own files (stages 5 and 6 add to them there)
    hand: crate::hand_tab::HandParams,
    rules: crate::rulebook::RulesParams,
) {
    let _frames = crate::frames::span(crate::frames::Sys::Panel);
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    // what in the panel goes by the month: the Markets panel month by month follows the
    // clock's month, and the news is listed again when more of it has come (the Hand
    // tab's list is made from the news, so it too)
    let month = match ui.tab {
        // (the Work half goes by the year only)
        Some(Tab::Markets) if ui.by_month && ui.markets != MarketsShow::Work => clock::series_month(tl.t),
        Some(Tab::News) | Some(Tab::Hand) => data.r.header.events.len(),
        _ => 0,
    };
    let key = (ui.tab, k, ui.news, ui.dirty, month);
    if ui.built == Some(key) {
        return;
    }
    // while the clock runs, a panel whose month alone has moved on is drawn again a few times a second at most
    let now = time.elapsed_secs();
    if tl.playing && ui.built.is_some_and(|b| (b.0, b.1, b.2, b.3) == (key.0, key.1, key.2, key.3)) && now - ui.drawn_at < 0.25 {
        return;
    }
    ui.drawn_at = now;
    ui.built = Some(key);
    let Some(tab) = ui.tab else { return };
    for mut t in &mut title {
        t.0 = tab.label(sandbox.is_some()).to_string();
    }
    let Ok((be, children, mut scroll)) = body.single_mut() else { return };
    if let Some(ch) = children {
        for c in ch.iter() {
            commands.entity(c).despawn();
        }
    }
    // another panel, or another choice of news, opens at its top
    if *last_list != Some((tab, ui.news)) {
        *last_list = Some((tab, ui.news));
        scroll.0.y = 0.0;
    }
    let f = font.0.clone();
    let r = &data.r;
    let fam = fam.as_deref();
    // the family is known as it stands today: its panels say so when the clock has been taken back
    let past = live.as_ref().is_some_and(|l| crate::game::now_of(k, data.latest(), l.seen) == crate::game::Now::Past);
    let accent = accent(fam, sandbox.is_some());
    // the news of the year in progress is listed as it comes
    let in_progress = live.is_some() && k >= data.latest() && r.filled < r.ny;
    let news_to = r.y0 + k as i32 + in_progress as i32;
    // the last month that is over, or the newest before it that the run has (a month that has
    // come may wait a moment to go into the run, and the goods would have no price to show)
    let by_month = ui.by_month.then(|| replay::month_reached(r, clock::series_month(tl.t).saturating_sub(1)));
    let stands = live.as_ref().map(|l| &l.standing).filter(|_| !past);
    commands.entity(be).with_children(|p| match tab {
        // (in a sandbox nobody's family is "Your family")
        Tab::News => news_panel(p, &f, r, news_to, ui.news, ui.news_rows, fam.and_then(|v| v.player).filter(|_| sandbox.is_none())),
        Tab::Markets => markets_panel(p, &f, &mut images, r, k, ui.markets, ui.good, by_month, accent, &rules),
        Tab::Family => family::family_panel(p, &f, fam, r, k, past, ui.expanded, ui.rival, stands),
        Tab::Holdings => family::holdings_panel(p, &f, fam, r, k, past),
        Tab::Hand => crate::hand_tab::panel(p, &f, r, k, &hand),
        Tab::Rules => crate::rulebook::panel(p, &f, &rules),
    });
}

pub fn kind_colour(kind: &str) -> Color {
    match kind {
        "discovery" | "first" => Color::srgb(0.75, 0.52, 0.12),
        "railway" | "rail_plan" => Color::srgb(0.25, 0.27, 0.32),
        "canal" | "canal_plan" => Color::srgb(0.22, 0.42, 0.52),
        "milestone" => Color::srgb(0.28, 0.55, 0.33),
        "crossing" => Color::srgb(0.16, 0.48, 0.53),
        "steep" => Color::srgb(0.48, 0.36, 0.62),
        "alarm" | "famine" | "dearth" => Color::srgb(0.70, 0.25, 0.22),
        "harvest" => Color::srgb(0.62, 0.56, 0.24),
        "town" | "region" => Color::srgb(0.55, 0.40, 0.23),
        "family" => Color::srgb(0.62, 0.30, 0.45),
        "emigration" => Color::srgb(0.30, 0.34, 0.56),
        // what the hand did in a sandbox, in its ink
        "hand" => crate::sandbox::HAND_INK,
        // what became of the player's own orders at a year's turn (`Toasts::say`): done, or not
        "orders" => Color::srgb(0.62, 0.30, 0.45),
        "orders_refused" => Color::srgb(0.70, 0.25, 0.22),
        // a note kept at the window (note.rs)
        "note" => crate::note::NOTE_INK,
        _ => Color::srgb(0.45, 0.45, 0.45),
    }
}

pub fn kind_label(kind: &str) -> &'static str {
    match kind {
        "discovery" => "Invention",
        "first" => "First output",
        "railway" => "Railway",
        "rail_plan" => "Railway scheme",
        "canal" => "Canal",
        "canal_plan" => "Canal scheme",
        "milestone" => "Milestone",
        "crossing" => "Turning point",
        "steep" => "Markets",
        "alarm" => "Hard times",
        "verdict" => "In hindsight",
        "famine" => "Hunger",
        "harvest" => "Harvest",
        "dearth" => "Dearth",
        "town" => "New town",
        "region" => "Town and country",
        "closure" => "Closure",
        "family" => "Families",
        "emigration" => "Emigration",
        "state" => "The state",
        "orders" | "orders_refused" => "Your orders",
        "hand" => "By hand",
        "note" => "Your note",
        _ => "News",
    }
}

fn chip(p: &mut ChildSpawnerCommands, c: Color) {
    p.spawn((Node { width: Val::Px(9.0), height: Val::Px(9.0), margin: UiRect::top(Val::Px(5.0)), border_radius: BorderRadius::all(Val::Px(5.0)), flex_shrink: 0.0, ..default() }, BackgroundColor(c)));
}

pub fn small_button<C: Component>(p: &mut ChildSpawnerCommands, f: &Handle<Font>, label: &str, marker: C) {
    p.spawn((button_node(10.0, 4.0), BackgroundColor(BTN), Interaction::default(), FocusPolicy::Block, Hoverable, marker)).with_children(|b| {
        b.spawn(text(f, label, 13.0, INK));
    });
}

/// What the foot of the news list says when there is more than it lists.
fn news_foot(listed: usize, n: usize) -> Option<String> {
    (n > listed).then(|| format!("The newest {listed} of {n} are listed."))
}

/// The news up to the end of `year`, newest first.
fn news_panel(p: &mut ChildSpawnerCommands, f: &Handle<Font>, r: &Replay, year: i32, show: NewsShow, rows: usize, player: Option<usize>) {
    let wanted = |e: &replay::Event| match show {
        NewsShow::Major => e.major(),
        NewsShow::All => true,
        NewsShow::Families => e.family.is_some(),
    };
    let n = r.header.events.iter().filter(|e| (e.year.floor() as i32) <= year && wanted(e)).count();
    p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, margin: UiRect::bottom(Val::Px(4.0)), ..default() }).with_children(|row| {
        row.spawn(text(f, &format!("{n} {} up to {year}", if n == 1 { "item" } else { "items" }), 13.0, MUTED));
        let label = match show {
            NewsShow::Major => "Showing major",
            NewsShow::All => "Showing all",
            NewsShow::Families => "Showing the families",
        };
        small_button(row, f, label, NewsFilter);
    });
    let mut shown = 0;
    for e in r.header.events.iter().rev() {
        if (e.year.floor() as i32) > year || !wanted(e) {
            continue;
        }
        if shown >= rows {
            break;
        }
        shown += 1;
        p.spawn(Node { column_gap: Val::Px(9.0), padding: UiRect::vertical(Val::Px(4.0)), border: UiRect::bottom(Val::Px(1.0)), ..default() })
            .insert(BorderColor::all(RULE))
            .with_children(|row| {
                // family news in the family's colour, the player's own marked as such
                let (colour, label) = match e.family {
                    Some(fi) if Some(fi) == player => (family::banner(fi), "Your family"),
                    Some(fi) => (family::banner(fi), kind_label(&e.kind)),
                    None => (kind_colour(&e.kind), kind_label(&e.kind)),
                };
                chip(row, colour);
                row.spawn(Node { flex_direction: FlexDirection::Column, flex_shrink: 1.0, ..default() }).with_children(|col| {
                    col.spawn(text(f, &format!("{}  \u{00b7}  {}", label, clock::news_month(e.year)), 12.0, colour));
                    col.spawn(text(f, &e.title, 15.5, INK));
                    if !e.text.is_empty() {
                        col.spawn(text(f, &e.text, 12.5, MUTED));
                    }
                });
            });
    }
    if shown == 0 {
        p.spawn(text(f, "Nothing yet. Press play.", 14.0, MUTED));
    }
    // the list stops somewhere: say where, and offer the rest
    if let Some(foot) = news_foot(shown, n) {
        p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, margin: UiRect::vertical(Val::Px(6.0)), ..default() }).with_children(|row| {
            row.spawn(text(f, &foot, 13.0, MUTED));
            small_button(row, f, "Show more", NewsMore);
        });
    }
}

/// A whole number with commas: 12,340.
pub fn thousands(v: f32) -> String {
    let digits = format!("{:.0}", v.abs());
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if v < 0.0 { format!("-{out}") } else { out }
}

/// A count in the few characters there are beside a chart: 37, 1,204, 412k, 1.2m.
fn short_count(v: f32) -> String {
    if v >= 999_500.0 { format!("{:.1}m", v / 1e6) } else if v >= 9_999.5 { format!("{:.0}k", v / 1e3) } else { thousands(v) }
}

/// A quantity that may be small: 0.42, 3.6, 1,204, 412k.
fn short_qty(v: f32) -> String {
    if v >= 99.95 { short_count(v) } else if v >= 9.995 { format!("{v:.1}") } else { format!("{v:.2}") }
}

/// Room for a chart inside the side panel.
const CHART_W: f32 = 342.0;

/// A small round dot of a line's colour, for the key above a chart.
fn key_dot(p: &mut ChildSpawnerCommands, c: Color) {
    p.spawn((Node { width: Val::Px(8.0), height: Val::Px(8.0), border_radius: BorderRadius::all(Val::Px(4.0)), flex_shrink: 0.0, ..default() }, BackgroundColor(c)));
}

/// The island as a whole, a chart each: its people, what a wage buys, what is
/// made a head, how many live in towns, the railway and the works.
fn island_charts(p: &mut ChildSpawnerCommands, f: &Handle<Font>, images: &mut Assets<Image>, r: &Replay, k: usize, by_month: Option<usize>, accent: Color) {
    let year = r.y0 + k as i32;
    let span = by_month.map(|m| MonthSpan::of(r, m));
    match &span {
        Some(sp) => p.spawn(text(f, &format!("The island as a whole, month by month over five years. The marker and the figures are {}.", sp.marked()), 13.0, MUTED)),
        None => p.spawn(text(f, &format!("The island as a whole, year by year. The marker and the figures are {year}."), 13.0, MUTED)),
    };
    let acc = accent.to_srgba().to_u8_array();
    let pct = |v: f32| format!("{:.0}%", v * 100.0);
    let plain = |v: f32| format!("{v:.2}");
    let km = |v: f32| format!("{} km", thousands(v));
    let people = |v: f32| thousands(v);
    type Fig<'a> = &'a dyn Fn(f32) -> String;
    let pounds = |v: f32| money(v);
    let rows: [(&str, &str, Fig, Fig); 7] = [
        ("pop", "People", &people, &short_count),
        ("real_wage", "What a wage buys (1750 = 1)", &plain, &plain),
        // what is made, linked, a head (docs/rules/measures.md): only in a run that keeps it
        ("made_linked", "What is made a head (1750 = 1)", &plain, &plain),
        ("urban_share", "Living in towns", &pct, &pct),
        ("rail_km", "Railway", &km, &short_count),
        ("firms", "Works and shops", &people, &short_count),
        // (the island's money: a rule of money set by hand shows here first, docs/designs/sandbox.md 6.6)
        ("money", "The island's money", &pounds, &pounds),
    ];
    // what a wage buys is the linked wage over its 1750 figure (the chained one in a run without
    // it); a run with neither (the port's rules) keeps the fixed basket of 1750, and says so
    let chained = crate::work_charts::true_wage(r, span.as_ref().map(|sp| (sp.first, sp.n)));
    let made = crate::work_charts::made_a_head(r, span.as_ref().map(|sp| (sp.first, sp.n)));
    for (name, label, now, side) in rows {
        let (vals, mark, first_year) = match &span {
            Some(sp) => (sp.values(r, name), sp.mark, sp.first_year),
            None => (replay::yearly(r, name), k, r.y0),
        };
        let (vals, label) = match (name, &chained) {
            ("real_wage", Some(c)) => (c.clone(), label),
            ("real_wage", None) => (vals, "What a wage buys of 1750's basket"),
            ("made_linked", _) => (made.clone().unwrap_or_default(), label),
            _ => (vals, label),
        };
        if !vals.iter().any(|v| v.is_finite()) {
            continue;
        }
        p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), margin: UiRect::top(Val::Px(8.0)), ..default() }).with_children(|col| {
            col.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Baseline, padding: UiRect::right(Val::Px(4.0)), ..default() }).with_children(|row| {
                row.spawn(text(f, label, 15.0, INK));
                row.spawn(text(f, &vals.get(mark).filter(|v| v.is_finite()).map(|&v| now(v)).unwrap_or_default(), 15.0, INK));
            });
            let c = crate::chart::Chart { width: CHART_W, height: 58.0, line: &vals, second: None, accent: [acc[0], acc[1], acc[2]], mark, years: vals.len(), first_year, months: span.is_some() };
            crate::chart::spawn(col, f, images, &c, side, side);
        });
    }
    // ages (docs/rules/ages.md): the people by age in the year on the clock, reckoned
    crate::ages_chart::spawn(p, f, images, r, k);
    // state (docs/rules/state.md): the rate on pay, schools and pensions, from 1900
    crate::purse_chart::spawn(p, f, images, r, k);
}

/// Five years of months along the bottom of a chart: the four calendar years
/// before the marked month's and that year itself, the months still to come
/// left bare.
pub struct MonthSpan {
    /// the place in the run of the first month shown (a January)
    pub first: usize,
    /// how many months are shown
    pub n: usize,
    /// the marked month, counted from the first shown: the last month over when the clock stands where it does
    pub mark: usize,
    pub first_year: i32,
}

impl MonthSpan {
    /// The span for a marked month `m`, given as its place in the run (0 is the first January).
    pub fn of(r: &Replay, m: usize) -> MonthSpan {
        let m = m.min(r.series_len.saturating_sub(1));
        let first = (m / 12).saturating_sub(4) * 12;
        MonthSpan { first, n: 60.min(r.series_len.saturating_sub(first)), mark: m - first, first_year: r.y0 + (first / 12) as i32 }
    }

    pub fn values(&self, r: &Replay, name: &str) -> Vec<f32> {
        replay::monthly(r, name, self.first, self.n)
    }

    /// "March 1784".
    pub fn marked(&self) -> String {
        format!("{} {}", clock::MONTHS[self.mark % 12], self.first_year + (self.mark / 12) as i32)
    }
}

/// A good's history, under its row: its price in ink and how much of it is
/// made in the family's colour, each with a scale of its own.
#[allow(clippy::too_many_arguments)]
fn good_chart(p: &mut ChildSpawnerCommands, f: &Handle<Font>, images: &mut Assets<Image>, r: &Replay, k: usize, by_month: Option<usize>, name: &str, accent: Color) {
    let span = by_month.map(|m| MonthSpan::of(r, m));
    let (mut price, made, k, first_year) = match &span {
        Some(sp) => (sp.values(r, &format!("price_{name}")), sp.values(r, &format!("out_{name}")), sp.mark, sp.first_year),
        None => (replay::yearly(r, &format!("price_{name}")), replay::yearly(r, &format!("out_{name}")), k, r.y0),
    };
    // the years before anyone made it have a price in the record that nothing was sold at (town gas
    // before the first gasworks): the line begins where the making does
    if span.is_none() {
        unsold_until_made(&mut price, &made);
    }
    let acc = accent.to_srgba().to_u8_array();
    let money = |v: f32| crate::overlay::coin(v);
    p.spawn((Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(4.0), padding: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(2.0), Val::Px(8.0)), margin: UiRect::bottom(Val::Px(4.0)), border: UiRect::bottom(Val::Px(1.0)), ..default() }, BorderColor::all(RULE))).with_children(|col| {
        col.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(6.0), padding: UiRect::left(Val::Px(6.0)), ..default() }).with_children(|row| {
            key_dot(row, INK);
            let now = price.get(k).filter(|v| v.is_finite()).map(|&v| money(v)).unwrap_or_else(|| "none sold".into());
            row.spawn((text(f, &format!("Price {now}"), 13.0, INK), Node { margin: UiRect::right(Val::Px(12.0)), ..default() }));
            key_dot(row, accent);
            let now = made.get(k).filter(|v| v.is_finite()).map(|&v| format!("{} a month", short_qty(v))).unwrap_or_else(|| "none".into());
            row.spawn(text(f, &format!("Made {now}"), 13.0, accent));
        });
        let c = crate::chart::Chart { width: CHART_W, height: 88.0, line: &price, second: Some(&made), accent: [acc[0], acc[1], acc[2]], mark: k, years: price.len(), first_year, months: span.is_some() };
        crate::chart::spawn(col, f, images, &c, &money, &short_qty);
    });
}

/// Leave out the prices of the years before any of a good was made, year by
/// year from the first: they are prices nothing was sold at. A run that does
/// not hold what was made (`made` empty) keeps them all.
fn unsold_until_made(price: &mut [f32], made: &[f32]) {
    if made.is_empty() {
        return;
    }
    let first = made.iter().position(|v| v.is_finite() && *v > 0.0).unwrap_or(price.len());
    for p in price.iter_mut().take(first) {
        *p = f32::NAN;
    }
}

/// `by_month` is the marked month's place in the run when the panel goes month by month.
#[allow(clippy::too_many_arguments)]
fn markets_panel(p: &mut ChildSpawnerCommands, f: &Handle<Font>, images: &mut Assets<Image>, r: &Replay, k: usize, show: MarketsShow, open: Option<usize>, by_month: Option<usize>, accent: Color, rules: &crate::rulebook::RulesParams) {
    // three halves, chosen by a chip: the goods, the island as a whole, or its work; and the
    // first two either by the year or by the month (the Work half's figures are whole years')
    p.spawn(Node { column_gap: Val::Px(5.0), margin: UiRect::bottom(Val::Px(2.0)), align_items: AlignItems::Center, ..default() }).with_children(|row| {
        for (label, s) in [("Goods", MarketsShow::Prices), ("The island", MarketsShow::Island), ("Work", MarketsShow::Work)] {
            family::chip_button(row, f, label, s == show, accent, MarketsChip(s));
        }
        row.spawn(Node { flex_grow: 1.0, ..default() });
        if show != MarketsShow::Work {
            for (label, months) in [("By year", false), ("By month", true)] {
                family::chip_button(row, f, label, months == by_month.is_some(), accent, MonthsChip(months));
            }
        }
    });
    match show {
        MarketsShow::Island => return island_charts(p, f, images, r, k, by_month, accent),
        MarketsShow::Work => {
            // pay by class is drawn at the dials the rules last told (a sandbox's), whose chips stand over it
            let dials = crate::work_charts::Dials::of(rules.known.as_deref());
            return crate::work_charts::panel(p, f, images, r, k, dials, &|p| crate::rulebook::dial_chips(p, f, rules));
        }
        MarketsShow::Prices => {}
    }
    let first_year = r.y0;
    match by_month {
        Some(m) => p.spawn(text(f, &format!("Prices in {} against {first_year}, and the change over the twelve months before. Click a good for its history.", MonthSpan::of(r, m).marked()), 13.0, MUTED)),
        None => p.spawn(text(f, &format!("Prices against {first_year}, and the change over the last ten years. Click a good for its history."), 13.0, MUTED)),
    };
    let back = k.saturating_sub(10);
    // the month the figures stand at: the marked month, or the December of the year on the clock
    let month_now = by_month.unwrap_or(k * 12 + 11);
    // the goods priced against the year they were first made, and that year
    let mut late_goods: Vec<(String, i32)> = Vec::new();
    for group in good_groups(&r.header.goods) {
        // a head over each kind of good, where the content says which a good is
        if let Some(kind) = group.kind {
            p.spawn((text(f, kind.heading(), 12.5, MUTED), Node { margin: UiRect::new(Val::Px(GOOD_PAD), Val::Px(0.0), Val::Px(4.0), Val::Px(0.0)), ..default() }));
        }
        // (a column of their own, so that the rows stand closer than the panel's other lines)
        p.spawn(Node { flex_direction: FlexDirection::Column, flex_shrink: 0.0, row_gap: Val::Px(GOOD_ROW_GAP), ..default() }).with_children(|p| {
            for gi in group.goods {
                let g = &r.header.goods[gi];
                let name = format!("price_{}", g.name);
                // a good first made after the first year is priced against the year (or the month) it was first
                // made, not against a price nothing was sold at; and its change goes back no further than that
                let since = replay::first_made(r, &g.name).unwrap_or(0);
                let late = since >= 12;
                // by the month: the month's price, and the price twelve months before it
                let figures = match by_month {
                    Some(m) => (replay::month(r, &name, m), if late { replay::month(r, &name, since) } else { replay::annual(r, &name, 0) }, replay::month(r, &name, m.saturating_sub(12).max(if late { since } else { 0 }))),
                    None => (replay::annual(r, &name, k), replay::annual(r, &name, since / 12), replay::annual(r, &name, back.max(since / 12))),
                };
                let (Some(now), Some(first), Some(then)) = figures else { continue };
                if !(first > 0.0 && now > 0.0) {
                    continue;
                }
                // a good no works has made yet has a price in the record that nothing was sold at:
                // it keeps its place in the list and says so, and has no history to open
                if !replay::made_by(r, &g.name, month_now) {
                    p.spawn(good_row_node()).with_children(|row| {
                        good_name(row, f, &g.label, MUTED);
                        row.spawn((text(f, "not made yet", 13.0, MUTED), TextLayout::default().with_no_wrap()));
                    });
                    continue;
                }
                if late {
                    late_goods.push((g.label.clone(), r.y0 + (since / 12) as i32));
                }
                let idx = now / first;
                let ch = if then > 0.0 { now / then - 1.0 } else { 0.0 };
                let is_open = open == Some(gi);
                p.spawn((good_row_node(), BackgroundColor(if is_open { PAPER_2 } else { Color::NONE }), Interaction::default(), FocusPolicy::Block, RowHover { open: is_open }, GoodRow(gi))).with_children(|row| {
                    good_name(row, f, &g.label, INK);
                    // a bar on a log scale from the 1750 price in the middle: dearer to the right, cheaper to the left
                    row.spawn((Node { width: Val::Px(GOOD_BAR_W), height: Val::Px(8.0), flex_shrink: 0.0, border_radius: BorderRadius::all(Val::Px(4.0)), ..default() }, BackgroundColor(RULE))).with_children(|bar| {
                        let x = (idx.ln() / 3.0_f32.ln()).clamp(-1.0, 1.0);
                        let (l, w) = if x >= 0.0 { (50.0, 50.0 * x) } else { (50.0 + 50.0 * x, -50.0 * x) };
                        bar.spawn((
                            Node { position_type: PositionType::Absolute, left: Val::Percent(l), width: Val::Percent(w.max(3.0)), top: Val::Px(0.0), bottom: Val::Px(0.0), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() },
                            BackgroundColor(if x >= 0.0 { DEARER } else { CHEAPER }),
                        ));
                        bar.spawn((Node { position_type: PositionType::Absolute, left: Val::Percent(50.0), width: Val::Px(2.0), top: Val::Px(-3.0), bottom: Val::Px(-3.0), margin: UiRect::left(Val::Px(-1.0)), border_radius: BorderRadius::all(Val::Px(1.0)), ..default() }, BackgroundColor(MUTED)));
                    });
                    row.spawn((text(f, &index_words(idx), 13.5, INK), TextLayout::default().with_no_wrap(), Node { width: Val::Px(GOOD_INDEX_W), flex_shrink: 0.0, ..default() }));
                    let s = if ch.abs() < 0.03 { "steady".to_string() } else if ch > 0.0 { format!("up {:.0}%", ch * 100.0) } else { format!("down {:.0}%", -ch * 100.0) };
                    row.spawn((text(f, &s, 13.0, MUTED), TextLayout::default().with_no_wrap(), Node { width: Val::Px(GOOD_CHANGE_W), flex_shrink: 0.0, ..default() }));
                });
                if is_open {
                    good_chart(p, f, images, r, k, by_month, &g.name, accent);
                }
            }
        });
    }
    if let Some(note) = late_note(&late_goods) {
        p.spawn((text(f, &note, 12.0, MUTED), Node { margin: UiRect::new(Val::Px(GOOD_PAD), Val::Px(0.0), Val::Px(4.0), Val::Px(0.0)), ..default() }));
    }
    // what investors are building now
    let year = r.y0 + k as i32;
    let mut by_method: std::collections::BTreeMap<usize, (usize, f32)> = Default::default();
    for firm in &r.header.firms {
        if firm.born > year - 5 && firm.born <= year {
            let roi = firm.reason.as_ref().and_then(|v| v.get("roi")).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let e = by_method.entry(firm.method).or_default();
            e.0 += 1;
            e.1 += roi;
        }
    }
    let mut rows: Vec<(usize, usize, f32)> = by_method.into_iter().map(|(m, (n, s))| (m, n, s / n.max(1) as f32)).collect();
    rows.sort_by(|a, b| biggest_first(a.2, b.2));
    p.spawn((text(f, "What investors are building", 17.0, INK), Node { margin: UiRect::top(Val::Px(14.0)), ..default() }));
    p.spawn(text(f, &format!("New works since {}, with the return their backers expected.", year - 4), 13.0, MUTED));
    if rows.is_empty() {
        p.spawn(text(f, "No new works in the last five years.", 14.0, MUTED));
    }
    let kinds = rows.len();
    p.spawn(Node { flex_direction: FlexDirection::Column, flex_shrink: 0.0, ..default() }).with_children(|p| {
        for (m, n, roi) in rows.into_iter().take(INVESTOR_ROWS) {
            let ml = &r.header.methods[m].label;
            p.spawn(Node { justify_content: JustifyContent::SpaceBetween, column_gap: Val::Px(8.0), padding: UiRect::vertical(Val::Px(2.0)), ..default() }).with_children(|row| {
                row.spawn((text(f, &format!("{ml}  \u{00d7}{n}"), 14.0, INK), TextLayout::default().with_no_wrap()));
                row.spawn((text(f, &format!("{:.0}% a year", roi * 100.0), 14.0, INK), TextLayout::default().with_no_wrap(), Node { flex_shrink: 0.0, ..default() }));
            });
        }
    });
    // the list stops somewhere: say where
    if let Some(foot) = investors_foot(kinds) {
        p.spawn(text(f, &foot, 13.0, MUTED));
    }
}

/// The columns of a good's row in the Markets panel. They come to the width
/// of the side panel's list (`GOODS_W`): the name has what the figures leave,
/// 161 px, which holds a label of 24 letters at its size (about 150), so a long name
/// ("Soap and chemicals", "Lime and cement") neither wraps its row into two
/// nor pushes a figure out of its column; the tests measure both with the
/// font itself.
const GOOD_NAME_W: f32 = GOODS_W - 2.0 * GOOD_PAD - GOOD_BAR_W - GOOD_INDEX_W - GOOD_CHANGE_W - 3.0 * GOOD_GAP;
const GOOD_NAME_SIZE: f32 = 14.0;
const GOOD_BAR_W: f32 = 50.0;
const GOOD_INDEX_W: f32 = 40.0;
/// "down 99%" is 61 px at its size
const GOOD_CHANGE_W: f32 = 66.0;
const GOOD_GAP: f32 = 6.0;
const GOOD_PAD: f32 = 6.0;
/// The room between one good's row and the next.
const GOOD_ROW_GAP: f32 = 1.0;
/// The width the side panel's list gives its rows: the card, less its padding and the room kept for the thumb.
const GOODS_W: f32 = 380.0 - 16.0 - 10.0 - THUMB_ROOM;

/// How many kinds of new works "What investors are building" lists.
const INVESTOR_ROWS: usize = 10;

fn good_row_node() -> Node {
    Node { align_items: AlignItems::Center, column_gap: Val::Px(GOOD_GAP), padding: UiRect::axes(Val::Px(GOOD_PAD), Val::Px(2.0)), border_radius: BorderRadius::all(Val::Px(8.0)), ..default() }
}

/// A good's name in its column: on one line whatever its length, cut at the column's edge if it is longer than the column.
fn good_name(row: &mut ChildSpawnerCommands, f: &Handle<Font>, label: &str, colour: Color) {
    row.spawn(Node { width: Val::Px(GOOD_NAME_W), flex_shrink: 0.0, overflow: Overflow::clip_x(), ..default() }).with_children(|col| {
        col.spawn((text(f, label, GOOD_NAME_SIZE, colour), TextLayout::default().with_no_wrap()));
    });
}

/// The goods in the groups the Markets panel lists them under.
struct GoodGroup {
    /// the head over the group; none for a list that is not grouped
    kind: Option<replay::GoodKind>,
    goods: Vec<usize>,
}

/// The goods under their three heads (from the land, for the works, for the
/// household), each group in the content's order, where the content at hand
/// says which every good is; otherwise one list in the content's order, as a
/// recorded run gets.
fn good_groups(goods: &[replay::Good]) -> Vec<GoodGroup> {
    if goods.is_empty() || goods.iter().any(|g| g.kind.is_none()) {
        return vec![GoodGroup { kind: None, goods: (0..goods.len()).collect() }];
    }
    replay::GoodKind::ALL
        .into_iter()
        .map(|kind| GoodGroup { kind: Some(kind), goods: (0..goods.len()).filter(|&g| goods[g].kind == Some(kind)).collect() })
        .filter(|group| !group.goods.is_empty())
        .collect()
}

/// A price against the first year's, in the few characters its column has:
/// two places, and three for a price that has fallen to under a tenth, which
/// two would round to nothing (steel in 1900 at 0.004 of its price of 1750).
fn index_words(idx: f32) -> String {
    if idx >= 0.095 { format!("{idx:.2}") } else { format!("{idx:.3}") }
}

/// The line under the goods for those first made after the first year, which
/// are priced against that year: "Town gas is priced against 1818, the first
/// year it was made." and "Steel and town gas are priced against the first
/// year each was made: 1786 and 1818."
fn late_note(late: &[(String, i32)]) -> Option<String> {
    match late {
        [] => None,
        [(label, year)] => Some(format!("{label} is priced against {year}, the first year it was made.")),
        _ => {
            let labels: Vec<String> = late.iter().enumerate().map(|(i, (l, _))| if i == 0 { l.clone() } else { l.to_lowercase() }).collect();
            let years: Vec<String> = late.iter().map(|(_, y)| y.to_string()).collect();
            Some(format!("{} are priced against the first year each was made: {}.", family::join_words(&labels), family::join_words(&years)))
        }
    }
}

/// What the foot of "What investors are building" says when there are more kinds of new works than it lists.
fn investors_foot(kinds: usize) -> Option<String> {
    (kinds > INVESTOR_ROWS).then(|| format!("The {} kinds with the best returns, of {kinds} built.", family::number_word(INVESTOR_ROWS)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sailing_is_news_of_its_own_kind() {
        // the people's rule files "emigration" under the game's rules (core/peopling.rs);
        // until 12 November 2026 it was listed as grey "News" here
        assert_eq!(kind_label("emigration"), "Emigration");
        assert_ne!(kind_colour("emigration"), kind_colour("no such kind"));
    }

    #[test]
    fn the_news_says_how_much_it_lists() {
        assert_eq!(news_foot(120, 973).as_deref(), Some("The newest 120 of 973 are listed."));
        assert_eq!(news_foot(240, 973).as_deref(), Some("The newest 240 of 973 are listed."));
        // everything listed: nothing to say, and no button
        assert_eq!(news_foot(88, 88), None);
        assert_eq!(news_foot(0, 0), None);
    }

    #[test]
    fn the_tabs_drop_their_words_only_when_the_bar_is_too_narrow() {
        // the cards about as they measure in a game: the family's 300, the clock's 720, the tabs' 400 with their words
        let (family, clock, tabs) = (300.0, 720.0, 400.0);
        // a window 1600 wide leaves 1572 between the margins, and one 1280 wide 1252
        assert!(!tabs_must_shrink(1572.0, family, clock, tabs));
        assert!(tabs_must_shrink(1252.0, family, clock, tabs));
        // room for the three and the two gaps between them is room enough
        assert!(!tabs_must_shrink(family + clock + tabs + 2.0 * TOP_GAP, family, clock, tabs));
        assert!(tabs_must_shrink(family + clock + tabs + 2.0 * TOP_GAP - 1.0, family, clock, tabs));
    }

    #[test]
    fn the_wheel_moves_the_list_at_once_from_its_foot() {
        // a list 10,860 pixels long, wheeled far past its foot
        let mut asked = 0.0;
        for _ in 0..400 {
            asked = scroll_to(asked, asked.min(10_860.0), -36.0);
        }
        // one step past the end at most, however long the wheel turned
        assert_eq!(asked, 10_896.0);
        // one step up shows on screen at once
        let up = scroll_to(asked, 10_860.0, 36.0);
        assert_eq!(up, 10_824.0);
        // and it stops at the top
        assert_eq!(scroll_to(20.0, 20.0, 36.0), 0.0);
    }

    #[test]
    fn sorting_the_biggest_first_survives_a_number_that_is_not_one() {
        let mut v = vec![(0, 3.0), (1, f32::NAN), (2, 12.5), (3, 0.0), (4, f32::INFINITY), (5, -1.0)];
        v.sort_by(|a, b| biggest_first(a.1, b.1));
        let order: Vec<usize> = v.iter().map(|x| x.0).collect();
        assert_eq!(order, [4, 2, 0, 3, 5, 1]);
    }

    #[test]
    fn the_thumb_shows_how_much_of_a_list_shows_and_where() {
        // all of it shows, or all but half a pixel: no thumb
        assert_eq!(thumb_span(500.0, 400.0, 0.0), None);
        assert_eq!(thumb_span(500.0, 500.4, 0.0), None);
        // a list twice its room: a thumb half the room, at the top, then at the foot
        assert_eq!(thumb_span(500.0, 1000.0, 0.0), Some((0.0, 0.5)));
        assert_eq!(thumb_span(500.0, 1000.0, 500.0), Some((0.5, 0.5)));
        // scrolled past the foot (the wheel can ask a step too far) it stays at the foot
        assert_eq!(thumb_span(500.0, 1000.0, 900.0), Some((0.5, 0.5)));
        // a very long list keeps a thumb that can be seen
        let (top, h) = thumb_span(500.0, 100_000.0, 99_500.0).unwrap();
        assert_eq!(h, 0.06);
        assert!((top - 0.94).abs() < 1e-6);
        // a list with no room yet (the first frame) has none
        assert_eq!(thumb_span(0.0, 300.0, 0.0), None);
    }

    #[test]
    fn a_price_index_keeps_three_places_only_when_two_would_round_it_away() {
        assert_eq!(index_words(1.0), "1.00");
        assert_eq!(index_words(12.345), "12.35");
        assert_eq!(index_words(0.096), "0.10");
        assert_eq!(index_words(0.094), "0.094");
        // steel in 1900 against 1750
        assert_eq!(index_words(0.0034), "0.003");
    }

    #[test]
    fn a_goods_history_begins_where_its_making_does() {
        let nan = f32::NAN;
        // town gas: nothing made for two years, then some, then a year when every gasworks stood idle
        let mut price = vec![2.4, 2.5, 3.1, 2.9, 3.0];
        unsold_until_made(&mut price, &[0.0, nan, 4.0, 0.0, 5.0]);
        assert!(price[0].is_nan() && price[1].is_nan());
        assert_eq!(price[2..], [3.1, 2.9, 3.0]);
        // never made: no line at all; a record without what was made: every price kept
        let mut never = vec![1.0, 1.0];
        unsold_until_made(&mut never, &[0.0, 0.0]);
        assert!(never.iter().all(|p| p.is_nan()));
        let mut kept = vec![1.0, 2.0];
        unsold_until_made(&mut kept, &[]);
        assert_eq!(kept, [1.0, 2.0]);
    }

    #[test]
    fn a_good_first_made_late_says_what_its_price_is_against() {
        assert_eq!(late_note(&[]), None);
        assert_eq!(late_note(&[("Town gas".into(), 1818)]).as_deref(), Some("Town gas is priced against 1818, the first year it was made."));
        assert_eq!(late_note(&[("Steel".into(), 1786), ("Town gas".into(), 1818)]).as_deref(), Some("Steel and town gas are priced against the first year each was made: 1786 and 1818."));
    }

    #[test]
    fn the_investors_list_says_where_it_stops() {
        assert_eq!(investors_foot(0), None);
        assert_eq!(investors_foot(INVESTOR_ROWS), None);
        assert_eq!(investors_foot(18).as_deref(), Some("The ten kinds with the best returns, of 18 built."));
    }

    fn goods(kinds: &[Option<replay::GoodKind>]) -> Vec<replay::Good> {
        kinds.iter().enumerate().map(|(i, &kind)| replay::Good { name: format!("g{i}"), label: format!("Good {i}"), kind, weight: None, price: None }).collect()
    }

    #[test]
    fn the_goods_go_under_three_heads_only_where_the_content_says_which_each_is() {
        use replay::GoodKind::*;
        // a played game: every good has its kind; each head keeps the content's order, and an empty head is left out
        let g = goods(&[Some(Land), Some(Household), Some(Land), Some(Household), Some(Land)]);
        let groups = good_groups(&g);
        let heads: Vec<Option<replay::GoodKind>> = groups.iter().map(|x| x.kind).collect();
        assert_eq!(heads, [Some(Land), Some(Household)]);
        assert_eq!(groups[0].goods, [0, 2, 4]);
        assert_eq!(groups[1].goods, [1, 3]);
        // a recorded run, or any good without its kind: one list in the content's order, with no head
        let g = goods(&[Some(Land), None, Some(Works)]);
        let groups = good_groups(&g);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].kind, None);
        assert_eq!(groups[0].goods, [0, 1, 2]);
        assert_eq!(good_groups(&[]).len(), 1);
    }

    /// The width of a line of the interface's font at `size` pixels, from the
    /// advances in the font file itself (kerning left out, which only narrows a
    /// line of this font).
    fn line_width(s: &str, size: f32) -> f32 {
        let font = include_bytes!("../assets/fonts/Fredoka.ttf");
        let u16_at = |o: usize| u16::from_be_bytes([font[o], font[o + 1]]) as usize;
        let u32_at = |o: usize| u32::from_be_bytes([font[o], font[o + 1], font[o + 2], font[o + 3]]) as usize;
        let table = |tag: &[u8; 4]| (0..u16_at(4)).map(|i| 12 + 16 * i).find(|&r| &font[r..r + 4] == tag).map(|r| u32_at(r + 8)).expect("a table the font must have");
        let per_em = u16_at(table(b"head") + 18) as f32;
        let n_metrics = u16_at(table(b"hhea") + 34);
        let hmtx = table(b"hmtx");
        let cmap = table(b"cmap");
        // the Windows Unicode map, kept in segments (format 4)
        let sub = (0..u16_at(cmap + 2)).map(|i| cmap + 4 + 8 * i).find(|&r| u16_at(r) == 3 && u16_at(r + 2) == 1).map(|r| cmap + u32_at(r + 4)).expect("a Unicode map");
        assert_eq!(u16_at(sub), 4, "the map is kept in segments");
        let segs = u16_at(sub + 6) / 2;
        let (ends, starts) = (sub + 14, sub + 16 + 2 * segs);
        let (deltas, ranges) = (starts + 2 * segs, starts + 4 * segs);
        let glyph = |c: usize| -> usize {
            let Some(i) = (0..segs).find(|&i| u16_at(ends + 2 * i) >= c) else { return 0 };
            if u16_at(starts + 2 * i) > c {
                return 0;
            }
            let delta = u16_at(deltas + 2 * i);
            match u16_at(ranges + 2 * i) {
                0 => (c + delta) & 0xFFFF,
                r => match u16_at(ranges + 2 * i + r + 2 * (c - u16_at(starts + 2 * i))) {
                    0 => 0,
                    g => (g + delta) & 0xFFFF,
                },
            }
        };
        let advance = |g: usize| u16_at(hmtx + 4 * g.min(n_metrics - 1)) as f32;
        s.chars().map(|c| advance(glyph(c as usize))).sum::<f32>() * size / per_em
    }

    #[test]
    fn a_goods_row_fits_across_the_panel_with_a_name_of_twenty_four_letters() {
        // the columns and the gaps between them come to no more than the panel's list gives a row
        let row = 2.0 * GOOD_PAD + GOOD_NAME_W + GOOD_BAR_W + GOOD_INDEX_W + GOOD_CHANGE_W + 3.0 * GOOD_GAP;
        assert!(row <= GOODS_W, "a good's row is {row} px across and the list gives it {GOODS_W}");
        // a name of 24 letters, and the longest of the content's (the measure is checked on a plain word first)
        assert!(line_width("Grain", GOOD_NAME_SIZE) > 25.0 && line_width("Grain", GOOD_NAME_SIZE) < 50.0, "{}", line_width("Grain", GOOD_NAME_SIZE));
        for name in ["Chemicals and fertiliser", "Soap and chemicals", "Lime and cement", "Machinery"] {
            let w = line_width(name, GOOD_NAME_SIZE);
            assert!(w <= GOOD_NAME_W, "{name} is {w:.0} px at {GOOD_NAME_SIZE} px and the column is {GOOD_NAME_W}");
        }
        // the figures: a price index up to a hundred times the first year's, a rise of up to ten thousand per
        // cent (rails in the 1750s, when nobody yet wanted any) and the widest fall
        assert!(line_width("99.99", 13.5) <= GOOD_INDEX_W, "{}", line_width("99.99", 13.5));
        assert!(line_width("up 9999%", 13.0) <= GOOD_CHANGE_W, "{}", line_width("up 9999%", 13.0));
        assert!(line_width("down 99%", 13.0) <= GOOD_CHANGE_W, "{}", line_width("down 99%", 13.0));
        assert!(line_width("not made yet", 13.0) <= GOOD_BAR_W + GOOD_GAP + GOOD_INDEX_W + GOOD_GAP + GOOD_CHANGE_W);
    }

    #[test]
    fn a_list_of_up_to_forty_goods_is_reached_whole_at_both_window_sizes() {
        use replay::GoodKind::*;
        // a row is a line of the name's size and its padding, with the gap between rows; a head is a smaller line
        let row = GOOD_NAME_SIZE * 1.2 + 4.0 + GOOD_ROW_GAP;
        let head = 12.5 * 1.2 + 4.0;
        // the side panel's list in a window 800 and 1000 high: the panel stands 86 from the top and 70 from the foot,
        // less its padding, its title and the Markets panel's own lines above the goods (chips and a note, about 110)
        for window in [800.0, 1000.0] {
            let room = window - 86.0 - 70.0 - 24.0 - 35.0 - 110.0;
            for n in 1..=40 {
                let kinds: Vec<Option<replay::GoodKind>> = (0..n).map(|i| Some([Land, Works, Household][i * 7 % 3])).collect();
                let groups = good_groups(&goods(&kinds));
                // every good once, under at most three heads
                let mut all: Vec<usize> = groups.iter().flat_map(|g| g.goods.iter().copied()).collect();
                all.sort_unstable();
                assert_eq!(all, (0..n).collect::<Vec<_>>());
                assert!(groups.len() <= 3);
                // with the investors' ten rows and their foot under them
                let long = groups.len() as f32 * head + n as f32 * row + 30.0 + 10.0 * 20.0 + 20.0;
                match thumb_span(room, long, long - room) {
                    // what does not show is reached by the wheel, and the thumb says there is more and ends at the foot
                    Some((top, h)) => assert!(long > room && (top + h - 1.0).abs() < 1e-4, "{n} goods at {window}"),
                    None => assert!(long <= room + 0.5),
                }
            }
        }
    }
}
