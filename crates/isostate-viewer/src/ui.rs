//! The game interface: a top bar with the family, the date and the speed; a
//! tabbed side panel (family, holdings, markets, news); and a timeline along
//! the bottom with the year's news marked on it.
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
}

impl Tab {
    /// For the command line: "markets", "news", ... or anything else for no panel.
    pub fn parse(s: &str) -> Option<Tab> {
        match s.to_ascii_lowercase().as_str() {
            "family" => Some(Tab::Family),
            "holdings" => Some(Tab::Holdings),
            "markets" => Some(Tab::Markets),
            "news" => Some(Tab::News),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Tab::Family => "Family",
            Tab::Holdings => "Holdings",
            Tab::Markets => "Markets",
            Tab::News => "News",
        }
    }
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
    built: Option<(Option<Tab>, usize, NewsShow, u64)>,
}

/// The News panel lists this many items, and as many again for each Show more.
const NEWS_ROWS: usize = 120;

impl Default for UiState {
    fn default() -> Self {
        UiState { tab: Some(Tab::News), news: NewsShow::Major, hover: false, dirty: 0, expanded: None, rival: None, news_rows: NEWS_ROWS, markets: MarketsShow::Prices, good: None, built: None }
    }
}

/// The Markets panel has two halves: the goods and their prices, and the island as a whole.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MarketsShow {
    Prices,
    Island,
}

/// Whether a word of a `--view` says what the Markets panel shows: "island",
/// or "open:GOOD" for the goods with one good's history open.
pub fn markets_word(word: &str) -> bool {
    let w = word.to_ascii_lowercase();
    w == "island" || w.starts_with("open:")
}

impl UiState {
    /// Set the Markets panel as a `markets_word` asks (for screenshots).
    pub fn show_markets(&mut self, word: &str, r: &Replay) {
        let w = word.to_ascii_lowercase();
        let (show, good) = match w.strip_prefix("open:") {
            Some(g) => (MarketsShow::Prices, r.header.goods.iter().position(|x| x.name.eq_ignore_ascii_case(g) || x.label.eq_ignore_ascii_case(g))),
            None => (MarketsShow::Island, self.good),
        };
        if self.markets != show || self.good != good {
            self.markets = show;
            self.good = good;
            self.dirty += 1;
        }
    }
}

#[derive(Component)]
pub struct TabButton(pub Tab);
#[derive(Component, Clone, Copy, PartialEq)]
pub enum SpeedButton {
    Pause,
    Speed(f32),
}
/// The clock's speeds, in years a second, under the labels on their buttons.
/// The labels count against the slowest: 1x is a year every two seconds and
/// 8x is eight times that, four years a second. (They do not count years a
/// second; a test holds each label to its multiple.)
pub const SPEEDS: [(&str, f32); 4] = [("1\u{00d7}", 0.5), ("2\u{00d7}", 1.0), ("4\u{00d7}", 2.0), ("8\u{00d7}", 4.0)];

/// The next of the buttons' speeds above or below `speed`, for the Up and Down
/// keys: the keys land on a button, so one of them is always lit.
pub fn step_speed(speed: f32, faster: bool) -> f32 {
    let (first, last) = (SPEEDS[0].1, SPEEDS[SPEEDS.len() - 1].1);
    if faster {
        SPEEDS.iter().map(|s| s.1).find(|&v| v > speed + 1e-3).unwrap_or(last)
    } else {
        SPEEDS.iter().rev().map(|s| s.1).find(|&v| v < speed - 1e-3).unwrap_or(first)
    }
}

#[derive(Component)]
pub struct DateText;
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
/// Clicking this opens a tab (the family card opens the family).
#[derive(Component)]
pub struct OpenTab(pub Tab);
/// A live game: work out one more year and show it.
#[derive(Component)]
pub struct NextYearButton;
/// A live game: says when the next year is being worked out.
#[derive(Component)]
pub struct StepNote;
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

pub fn plugin(app: &mut App) {
    app.init_resource::<UiState>().add_systems(PostStartup, setup_hud).add_systems(
        Update,
        (button_colours, row_colours, hotkeys, tab_clicks, speed_clicks, news_filter_click, news_more_click, markets_clicks, timeline_drag, active_styles, topbar_text, panel_visibility, panel_rebuild, panel_scroll, timeline_knob, timeline_ticks).chain(),
    );
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

pub fn setup_hud(mut commands: Commands, font: Res<UiFont>, data: Res<Data>, fam: Option<Res<FamilyView>>, icons: Res<crate::icons::Icons>) {
    let f = &font.0;
    let crest_col = fam.as_ref().and_then(|v| v.player()).map(|p| p.colour).unwrap_or(Color::srgb(0.55, 0.47, 0.38));
    let family_name = fam.as_ref().and_then(|v| v.player()).map(|p| format!("The {} family", p.name)).unwrap_or_else(|| "A recorded history".into());

    // three cards across the top: the family, the date and the clock, the tabs
    commands
        .spawn((
            Node { position_type: PositionType::Absolute, left: Val::Px(14.0), right: Val::Px(14.0), top: Val::Px(12.0), justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Start, column_gap: Val::Px(12.0), ..default() },
            GlobalZIndex(10),
        ))
        .with_children(|top| {
            // the side cards take what their contents need, the clock the rest, so a narrower window still fits
            top.spawn(Node { flex_shrink: 0.0, ..default() }).with_children(|wrap| {
                wrap.spawn((
                    card(Node {
                        height: Val::Px(62.0),
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(12.0),
                        border_radius: BorderRadius::all(Val::Px(16.0)),
                        ..default()
                    }),
                    OpenTab(Tab::Family),
                ))
                .with_children(|left| {
                    // a little shield with the family's initial
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
                        let letter = family_name.trim_start_matches("The ").chars().next().unwrap_or('A').to_string();
                        c.spawn((text(f, &letter, 19.0, Color::WHITE), CrestLetter));
                    });
                    left.spawn(Node { flex_direction: FlexDirection::Column, ..default() }).with_children(|col| {
                        col.spawn((text(f, &family_name, 19.0, INK), FamilyNameText));
                        col.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(4.0), ..default() }).with_children(|row| {
                            let slot = |row: &mut ChildSpawnerCommands, i: usize, gap: bool| {
                                row.spawn((text(f, "", 13.5, MUTED), FortuneSlot(i), Node { margin: UiRect::right(Val::Px(if gap { 8.0 } else { 0.0 })), ..default() }));
                            };
                            if fam.is_some() {
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
                wrap.spawn(card(Node {
                    height: Val::Px(62.0),
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(14.0),
                    border_radius: BorderRadius::all(Val::Px(16.0)),
                    ..default()
                }))
                .with_children(|mid| {
                    // wide enough for the note under the year, so the buttons do not shift when it comes and goes
                    mid.spawn(Node { flex_direction: FlexDirection::Column, min_width: Val::Px(118.0), ..default() }).with_children(|col| {
                        col.spawn((text(f, "", 25.0, INK), TextLayout::default().with_no_wrap(), DateText));
                        if data.live {
                            col.spawn((text(f, "", 12.0, MUTED), TextLayout::default().with_no_wrap(), StepNote, Node { margin: UiRect::top(Val::Px(-3.0)), ..default() }));
                        }
                    });
                    mid.spawn(Node { column_gap: Val::Px(4.0), ..default() }).with_children(|row| {
                        for (label, b) in std::iter::once(("Pause", SpeedButton::Pause)).chain(SPEEDS.iter().map(|&(label, v)| (label, SpeedButton::Speed(v)))) {
                            row.spawn((button_node(10.0, 5.0), BackgroundColor(BTN), Interaction::default(), FocusPolicy::Block, Hoverable, b)).with_children(|bt| {
                                bt.spawn((text(f, label, 14.0, INK), TextLayout::default().with_no_wrap()));
                            });
                        }
                        if data.live {
                            // one year on, then stop: the natural rhythm of a turn
                            row.spawn((Node { margin: UiRect::left(Val::Px(6.0)), ..button_node(10.0, 5.0) }, BackgroundColor(BTN), Interaction::default(), FocusPolicy::Block, Hoverable, NextYearButton))
                                .with_children(|bt| {
                                    bt.spawn((text(f, "Next year", 14.0, INK), TextLayout::default().with_no_wrap()));
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
            top.spawn(Node { flex_shrink: 0.0, justify_content: JustifyContent::End, ..default() }).with_children(|wrap| {
                wrap.spawn(card(Node {
                    height: Val::Px(62.0),
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    border_radius: BorderRadius::all(Val::Px(16.0)),
                    ..default()
                }))
                .with_children(|right| {
                    for t in [Tab::Family, Tab::Holdings, Tab::Markets, Tab::News] {
                        let ic = match t {
                            Tab::Family => &icons.family,
                            Tab::Holdings => &icons.holdings,
                            Tab::Markets => &icons.markets,
                            Tab::News => &icons.news,
                        };
                        right.spawn((Node { column_gap: Val::Px(6.0), ..button_node(11.0, 7.0) }, BackgroundColor(BTN), Interaction::default(), FocusPolicy::Block, Hoverable, TabButton(t))).with_children(|bt| {
                            bt.spawn(crate::icons::icon(ic, 16.0, INK));
                            bt.spawn(text(f, t.label(), 15.0, INK));
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
            p.spawn((text(f, "News", 22.0, INK), PanelTitle, Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() }));
            p.spawn((
                Node { flex_direction: FlexDirection::Column, flex_grow: 1.0, min_height: Val::Px(0.0), overflow: Overflow::scroll_y(), row_gap: Val::Px(6.0), padding: UiRect::right(Val::Px(6.0)), ..default() },
                ScrollPosition::default(),
                PanelBody,
            ));
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
            wrap.spawn(card(Node { flex_direction: FlexDirection::Column, padding: UiRect::axes(Val::Px(26.0), Val::Px(20.0)), row_gap: Val::Px(7.0), border_radius: BorderRadius::all(Val::Px(18.0)), ..default() })).with_children(|c| {
                c.spawn((text(f, "How to look around", 22.0, INK), Node { margin: UiRect::bottom(Val::Px(6.0)), ..default() }));
                for (key, what) in [
                    ("Drag", "move across the island"),
                    ("Right-drag", "turn and tilt the view"),
                    ("Scroll", "come closer or stand back"),
                    ("W A S D", "walk the view, Q and E to turn"),
                    ("Click", "ask about a works, a town or a square"),
                    ("Next year", "in a game: play one year, then stop"),
                    ("Space", "start or stop the years"),
                    ("Left, Right", "one year back or on"),
                    ("Up, Down", "faster or slower"),
                    ("1 2 3 4", "family, holdings, markets, news"),
                    ("M", "map views: rent, markets, reach, prices, wages"),
                    ("G", "the next good, when prices are mapped"),
                    ("T", "miniature focus on or off"),
                    ("Esc", "close the note about what you clicked"),
                ] {
                    c.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                        row.spawn(Node { width: Val::Px(120.0), justify_content: JustifyContent::End, ..default() }).with_children(|kc| keycap(kc, f, key));
                        row.spawn(text(f, what, 15.0, INK));
                    });
                }
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
                    BackgroundColor(Color::srgb(0.80, 0.74, 0.64)),
                    TimelineFill,
                ));
                // a tick for each major piece of news, filled in by timeline_ticks
                track.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(0.0), right: Val::Px(0.0), top: Val::Px(0.0), bottom: Val::Px(0.0), ..default() }, TickLayer));
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

fn hotkeys(keys: Res<ButtonInput<KeyCode>>, mut ui: ResMut<UiState>, mut help: Query<&mut Visibility, With<HelpCard>>, cards: Res<crate::game::CardOpen>) {
    // the panels and the help wait while a card wants an answer
    if cards.0 {
        return;
    }
    for (k, t) in [(KeyCode::Digit1, Tab::Family), (KeyCode::Digit2, Tab::Holdings), (KeyCode::Digit3, Tab::Markets), (KeyCode::Digit4, Tab::News)] {
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
) {
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

/// Selected tabs and the current speed are shown in the family's colour.
fn active_styles(
    ui: Res<UiState>,
    tl: Res<Timeline>,
    fam: Option<Res<FamilyView>>,
    tabs: Query<(&TabButton, &Interaction, &Children, Entity), Without<SpeedButton>>,
    speeds: Query<(&SpeedButton, &Interaction, &Children, Entity), Without<TabButton>>,
    mut bgs: Query<&mut BackgroundColor>,
    mut cols: Query<&mut TextColor>,
    mut imgs: Query<&mut ImageNode>,
) {
    let accent = fam.as_ref().and_then(|v| v.player()).map(|p| p.colour).unwrap_or(Color::srgb(0.55, 0.42, 0.30));
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

fn timeline_drag(mut tl: ResMut<Timeline>, data: Res<Data>, q: Query<(&Interaction, &RelativeCursorPosition), With<TimelineTrack>>) {
    for (i, rel) in &q {
        if *i != Interaction::Pressed {
            continue;
        }
        if let Some(n) = rel.normalized {
            let last = data.r.ny.max(1) - 1;
            let x = (n.x + 0.5).clamp(0.0, 1.0);
            tl.playing = false;
            tl.t = (x * last as f32).round().min(data.latest() as f32);
        }
    }
}

fn timeline_knob(tl: Res<Timeline>, data: Res<Data>, mut q: Query<&mut Node, (With<TimelineKnob>, Without<TimelineFill>)>, mut fill: Query<&mut Node, (With<TimelineFill>, Without<TimelineKnob>)>) {
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
}

/// News ticks along the timeline, redrawn as a live game adds news.
fn timeline_ticks(mut commands: Commands, data: Res<Data>, layer: Query<(Entity, Option<&Children>), With<TickLayer>>, mut drawn: Local<Option<(usize, usize)>>) {
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
    commands.entity(e).with_children(|track| {
        for e in r.header.events.iter().filter(|e| e.major()) {
            let x = ((e.year - r.y0 as f32) / span * 100.0).clamp(0.0, 100.0);
            let h = 6.0 + 10.0 * ((e.weight - 0.6) / 0.4).clamp(0.0, 1.0);
            track.spawn((
                Node { position_type: PositionType::Absolute, left: Val::Percent(x), top: Val::Px(16.0 - h), width: Val::Px(2.0), height: Val::Px(h), border_radius: BorderRadius::all(Val::Px(1.0)), ..default() },
                BackgroundColor(kind_colour(&e.kind).with_alpha(0.8)),
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

fn topbar_text(
    tl: Res<Timeline>,
    data: Res<Data>,
    sm: Option<Res<crate::SeasonMats>>,
    clock: Res<crate::SeasonClock>,
    fam: Option<Res<FamilyView>>,
    live: Option<Res<crate::live::Live>>,
    mut date: Query<&mut Text, (With<DateText>, Without<FortuneSlot>, Without<FamilyNameText>)>,
    mut slots: Query<(&mut Text, &FortuneSlot), (Without<DateText>, Without<FamilyNameText>)>,
    mut name: Query<&mut Text, (With<FamilyNameText>, Without<DateText>, Without<FortuneSlot>)>,
) {
    let r = &data.r;
    let k = (tl.t.floor() as usize).min(r.ny - 1);
    let year = r.y0 + k as i32;
    // the season is named only when it is the date's: the years going slowly
    // enough for the seasons to keep up, or a still of a chosen season. The
    // name is of the same phase the ground, the trees and the sun are showing.
    let forced = sm.as_ref().and_then(|m| m.forced);
    let s = if forced.is_some() || clock.follows { format!("{} {}", season_name(crate::season_phase(&clock, forced)), year) } else { format!("{year}") };
    for mut t in &mut date {
        if t.0 != s {
            t.0 = s.clone();
        }
    }
    let vals: Vec<String> = match fam.as_ref().and_then(|v| v.player()) {
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

fn panel_scroll(ui: Res<UiState>, scroll: Res<AccumulatedMouseScroll>, panel: Query<&RelativeCursorPosition, With<SidePanel>>, mut body: Query<(&mut ScrollPosition, &ComputedNode), With<PanelBody>>) {
    if ui.tab.is_none() || !panel.iter().any(|p| p.cursor_over()) {
        return;
    }
    let dy = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y * 36.0,
        MouseScrollUnit::Pixel => scroll.delta.y,
    };
    if dy != 0.0 {
        for (mut s, c) in &mut body {
            scroll_panel(&mut s, c, dy);
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
) {
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    let key = (ui.tab, k, ui.news, ui.dirty);
    if ui.built == Some(key) {
        return;
    }
    ui.built = Some(key);
    let Some(tab) = ui.tab else { return };
    for mut t in &mut title {
        t.0 = tab.label().to_string();
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
    let past = live.is_some_and(|l| crate::game::now_of(k, data.latest(), l.seen) == crate::game::Now::Past);
    let accent = fam.and_then(|v| v.player()).map(|p| p.colour).unwrap_or(Color::srgb(0.55, 0.42, 0.30));
    commands.entity(be).with_children(|p| match tab {
        Tab::News => news_panel(p, &f, r, k, ui.news, ui.news_rows, fam.and_then(|v| v.player)),
        Tab::Markets => markets_panel(p, &f, &mut images, r, k, ui.markets, ui.good, accent),
        Tab::Family => family::family_panel(p, &f, fam, r, k, past, ui.expanded, ui.rival),
        Tab::Holdings => family::holdings_panel(p, &f, fam, r, k, past),
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

fn news_panel(p: &mut ChildSpawnerCommands, f: &Handle<Font>, r: &Replay, k: usize, show: NewsShow, rows: usize, player: Option<usize>) {
    let year = r.y0 + k as i32;
    let wanted = |e: &replay::Event| match show {
        NewsShow::Major => e.major(),
        NewsShow::All => true,
        NewsShow::Families => e.family.is_some(),
    };
    let n = r.header.events.iter().filter(|e| (e.year.floor() as i32) <= year && wanted(e)).count();
    p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, margin: UiRect::bottom(Val::Px(4.0)), ..default() }).with_children(|row| {
        row.spawn(text(f, &format!("{n} items up to {year}"), 13.0, MUTED));
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
                    col.spawn(text(f, &format!("{}  \u{00b7}  {}", label, e.year.floor() as i32), 12.0, colour));
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

/// The island as a whole, a chart each: its people, what a wage buys, how many
/// live in towns, the railway and the works.
fn island_charts(p: &mut ChildSpawnerCommands, f: &Handle<Font>, images: &mut Assets<Image>, r: &Replay, k: usize, accent: Color) {
    let year = r.y0 + k as i32;
    p.spawn(text(f, &format!("The island as a whole, year by year. The marker and the figures are {year}."), 13.0, MUTED));
    let acc = accent.to_srgba().to_u8_array();
    let pct = |v: f32| format!("{:.0}%", v * 100.0);
    let plain = |v: f32| format!("{v:.2}");
    let km = |v: f32| format!("{} km", thousands(v));
    let people = |v: f32| thousands(v);
    type Fig<'a> = &'a dyn Fn(f32) -> String;
    let rows: [(&str, &str, Fig, Fig); 5] = [
        ("pop", "People", &people, &short_count),
        ("real_wage", "What a wage buys (1750 = 1)", &plain, &plain),
        ("urban_share", "Living in towns", &pct, &pct),
        ("rail_km", "Railway", &km, &short_count),
        ("firms", "Works and shops", &people, &short_count),
    ];
    for (name, label, now, side) in rows {
        let vals = replay::yearly(r, name);
        if !vals.iter().any(|v| v.is_finite()) {
            continue;
        }
        p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), margin: UiRect::top(Val::Px(8.0)), ..default() }).with_children(|col| {
            col.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Baseline, padding: UiRect::right(Val::Px(4.0)), ..default() }).with_children(|row| {
                row.spawn(text(f, label, 15.0, INK));
                row.spawn(text(f, &vals.get(k).filter(|v| v.is_finite()).map(|&v| now(v)).unwrap_or_default(), 15.0, INK));
            });
            let c = crate::chart::Chart { width: CHART_W, height: 58.0, line: &vals, second: None, accent: [acc[0], acc[1], acc[2]], mark: k, years: vals.len(), first_year: r.y0 };
            crate::chart::spawn(col, f, images, &c, side, side);
        });
    }
}

/// A good's history, under its row: its price in ink and how much of it is
/// made in the family's colour, each with a scale of its own.
fn good_chart(p: &mut ChildSpawnerCommands, f: &Handle<Font>, images: &mut Assets<Image>, r: &Replay, k: usize, name: &str, accent: Color) {
    let price = replay::yearly(r, &format!("price_{name}"));
    let made = replay::yearly(r, &format!("out_{name}"));
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
        let c = crate::chart::Chart { width: CHART_W, height: 88.0, line: &price, second: Some(&made), accent: [acc[0], acc[1], acc[2]], mark: k, years: price.len(), first_year: r.y0 };
        crate::chart::spawn(col, f, images, &c, &money, &short_qty);
    });
}

#[allow(clippy::too_many_arguments)]
fn markets_panel(p: &mut ChildSpawnerCommands, f: &Handle<Font>, images: &mut Assets<Image>, r: &Replay, k: usize, show: MarketsShow, open: Option<usize>, accent: Color) {
    // two halves, chosen by a chip: the goods, or the island as a whole
    p.spawn(Node { column_gap: Val::Px(5.0), margin: UiRect::bottom(Val::Px(2.0)), ..default() }).with_children(|row| {
        for (label, s) in [("Goods", MarketsShow::Prices), ("The island", MarketsShow::Island)] {
            family::chip_button(row, f, label, s == show, accent, MarketsChip(s));
        }
    });
    if show == MarketsShow::Island {
        return island_charts(p, f, images, r, k, accent);
    }
    p.spawn(text(f, "Prices against 1750, and the change over the last ten years. Click a good for its history.", 13.0, MUTED));
    let back = k.saturating_sub(10);
    for (gi, g) in r.header.goods.iter().enumerate() {
        let name = format!("price_{}", g.name);
        let (Some(now), Some(first), Some(then)) = (replay::annual(r, &name, k), replay::annual(r, &name, 0), replay::annual(r, &name, back)) else { continue };
        if !(first > 0.0 && now > 0.0) {
            continue;
        }
        let idx = now / first;
        let ch = if then > 0.0 { now / then - 1.0 } else { 0.0 };
        let is_open = open == Some(gi);
        p.spawn((
            Node { align_items: AlignItems::Center, column_gap: Val::Px(8.0), padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)), border_radius: BorderRadius::all(Val::Px(8.0)), ..default() },
            BackgroundColor(if is_open { PAPER_2 } else { Color::NONE }),
            Interaction::default(),
            FocusPolicy::Block,
            RowHover { open: is_open },
            GoodRow(gi),
        ))
        .with_children(|row| {
            row.spawn((text(f, &g.label, 14.5, INK), Node { width: Val::Px(92.0), ..default() }));
            // a bar on a log scale from the 1750 price in the middle: dearer to the right, cheaper to the left
            row.spawn((Node { width: Val::Px(110.0), height: Val::Px(8.0), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() }, BackgroundColor(RULE))).with_children(|bar| {
                let x = (idx.ln() / 3.0_f32.ln()).clamp(-1.0, 1.0);
                let (l, w) = if x >= 0.0 { (50.0, 50.0 * x) } else { (50.0 + 50.0 * x, -50.0 * x) };
                bar.spawn((
                    Node { position_type: PositionType::Absolute, left: Val::Percent(l), width: Val::Percent(w.max(1.5)), top: Val::Px(0.0), bottom: Val::Px(0.0), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() },
                    BackgroundColor(if x >= 0.0 { DEARER } else { CHEAPER }),
                ));
                bar.spawn((Node { position_type: PositionType::Absolute, left: Val::Percent(50.0), width: Val::Px(2.0), top: Val::Px(-3.0), bottom: Val::Px(-3.0), margin: UiRect::left(Val::Px(-1.0)), border_radius: BorderRadius::all(Val::Px(1.0)), ..default() }, BackgroundColor(MUTED)));
            });
            row.spawn((text(f, &format!("{:.2}", idx), 13.5, INK), Node { width: Val::Px(40.0), ..default() }));
            let s = if ch.abs() < 0.03 { "steady".to_string() } else if ch > 0.0 { format!("up {:.0}%", ch * 100.0) } else { format!("down {:.0}%", -ch * 100.0) };
            row.spawn((text(f, &s, 13.0, MUTED), Node { width: Val::Px(64.0), ..default() }));
        });
        if is_open {
            good_chart(p, f, images, r, k, &g.name, accent);
        }
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
    for (m, n, roi) in rows.into_iter().take(10) {
        let ml = &r.header.methods[m].label;
        p.spawn(Node { justify_content: JustifyContent::SpaceBetween, padding: UiRect::vertical(Val::Px(2.0)), ..default() }).with_children(|row| {
            row.spawn(text(f, &format!("{ml}  \u{00d7}{n}"), 14.0, INK));
            row.spawn(text(f, &format!("{:.0}% a year", roi * 100.0), 14.0, INK));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_news_says_how_much_it_lists() {
        assert_eq!(news_foot(120, 973).as_deref(), Some("The newest 120 of 973 are listed."));
        assert_eq!(news_foot(240, 973).as_deref(), Some("The newest 240 of 973 are listed."));
        // everything listed: nothing to say, and no button
        assert_eq!(news_foot(88, 88), None);
        assert_eq!(news_foot(0, 0), None);
    }

    #[test]
    fn the_speed_keys_land_on_the_buttons() {
        // up and down through the buttons, stopping at the ends
        assert_eq!(step_speed(0.5, true), 1.0);
        assert_eq!(step_speed(1.0, true), 2.0);
        assert_eq!(step_speed(2.0, true), 4.0);
        assert_eq!(step_speed(4.0, true), 4.0);
        assert_eq!(step_speed(4.0, false), 2.0);
        assert_eq!(step_speed(1.0, false), 0.5);
        assert_eq!(step_speed(0.5, false), 0.5);
        // from a speed between two buttons (as the keys used to leave it), onto the next one
        assert_eq!(step_speed(0.75, true), 1.0);
        assert_eq!(step_speed(3.0, false), 2.0);
        assert_eq!(step_speed(20.0, false), 4.0);
        // whatever the keys do, a button has that speed
        let mut v = 2.0;
        for faster in [true, true, true, false, false, false, false, false, true] {
            v = step_speed(v, faster);
            assert!(SPEEDS.iter().any(|s| s.1 == v), "{v} is no button's speed");
        }
    }

    #[test]
    fn the_speed_labels_are_multiples_of_the_slowest() {
        for (label, v) in SPEEDS {
            let times: f32 = label.trim_end_matches('\u{00d7}').parse().unwrap();
            assert_eq!(v / SPEEDS[0].1, times, "{label} is {} times 1x", v / SPEEDS[0].1);
        }
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
}
