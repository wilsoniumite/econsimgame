//! The card a click on a firm opens, as terms and numbers (docs/designs/words.md).
//!
//! Under the firm's name, one line: its method (a term, whose tooltip is made
//! from the method's own recipe in the record), its market town (a thing on
//! the island) and the year it was founded. Then rows of a term and a number:
//! its capacity and size, the return its founders expected; under the good's
//! own name ("Per cloth"), the price it sold at and each cost of a unit as an
//! amount times a price, and the margin left; the ground rent it bid and the
//! use it outbid; its owner. Every sentence the card used to carry is in the
//! tooltip of a row, as the glossary's sentences and the row's parts.
//!
//! In a live sandbox the simulation's rows follow (`inspect::tell`, the
//! core's inspect.rs, asked by inspector.rs), each a term of the glossary and
//! a figure, under the months they are of. The line a map view adds to a card
//! (`MapOverlay::said`) stands under them all, as it is.
//!
//! The card is laid out from a plan (`plan`), which is compared with the one
//! drawn, so that a card is drawn again only when what it shows has changed.

use std::sync::Arc;

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use isostate_core::inspect::{Thing, Told};

use crate::family::FamilyView;
use crate::replay::{self, Replay};
use crate::tips::{self, row, LitRow, Tip, TipBody};
use crate::ui::{INK, MUTED, RULE};
use crate::words::{self, plain, Glossary, Link, Piece};
use crate::{Data, Pick, Picked, Timeline, UiFont};

/// Where the firm's card is drawn in the note about what was clicked.
#[derive(Component)]
pub struct FirmCard;

/// A row of the card: its label, a small figure of how it is made (an amount
/// times a price, a size), its value, and its tooltip.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub label: Vec<Piece>,
    pub detail: Option<String>,
    pub value: Vec<Piece>,
    pub tip: TipBody,
    /// what tells this row from the others on the card: its section and its label
    pub id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    /// a small heading in capitals: the good's name, or when the figures are of
    pub heading: Option<String>,
    pub lines: Vec<Line>,
}

/// The whole card, but for its title, which is the firm's name.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Plan {
    pub top: Vec<Piece>,
    pub sections: Vec<Section>,
    /// the map view's line, as it is
    pub foot: Option<String>,
}

impl Plan {
    /// Every word on the card itself, and every word in its rows' tooltips.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn words(&self) -> (Vec<String>, Vec<String>) {
        let text = |p: &[Piece]| p.iter().map(|x| x.text()).collect::<String>();
        let mut card = vec![text(&self.top)];
        let mut tipped = Vec::new();
        for s in &self.sections {
            card.extend(s.heading.clone());
            for l in &s.lines {
                card.push(text(&l.label));
                card.extend(l.detail.clone());
                card.push(text(&l.value));
                tipped.push(l.tip.words());
            }
        }
        (card, tipped)
    }
}

// ------------------------------------------------------------------ figures

/// Money with its sign: pence below a thousand, whole pounds with their thousands marked above.
pub fn money(x: f64) -> String {
    let a = x.abs();
    let s = if a >= 1000.0 { thousands(a.round() as u64) } else { format!("{a:.2}") };
    if x < 0.0 && a >= 0.005 { format!("\u{2212}\u{a3}{s}") } else { format!("\u{a3}{s}") }
}

/// A count with its thousands marked.
pub fn thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A quantity of a good: whole from ten up, else to three figures.
pub fn qty(x: f64) -> String {
    let a = x.abs();
    if a >= 10.0 {
        thousands(a.round() as u64)
    } else if a >= 1.0 {
        format!("{a:.2}")
    } else {
        format!("{a:.3}")
    }
}

/// A multiple, as `×20.8`.
pub fn times(x: f64) -> String {
    if x >= 10.0 { format!("\u{d7}{x:.1}") } else if x >= 1.0 { format!("\u{d7}{x:.2}") } else { format!("\u{d7}{x:.3}") }
}

/// The costs of a unit rounded to pence so that they add up, with the margin,
/// to the price shown: each rounded, then what is left over put on the
/// largest. The figures are worked from the unrounded amounts and prices.
pub fn pence_that_add_up(price: f64, margin: f64, costs: &[f64]) -> Vec<i64> {
    let pence = |x: f64| (x * 100.0).round() as i64;
    let mut out: Vec<i64> = costs.iter().map(|&c| pence(c)).collect();
    let short = pence(price) - pence(margin) - out.iter().sum::<i64>();
    if let Some(i) = (0..costs.len()).max_by(|&a, &b| costs[a].abs().total_cmp(&costs[b].abs())) {
        out[i] += short;
    }
    out
}

fn pence_money(p: i64) -> String {
    money(p as f64 / 100.0)
}

fn good_label(r: &Replay, g: usize) -> String {
    r.header.goods.get(g).map_or_else(|| format!("good {g}"), |x| x.label.clone())
}

/// The land use a firm outbid, as the glossary names it where it has a term.
fn outbid(displaced: &str) -> Vec<Piece> {
    let (key, word) = match displaced {
        "urban" => (Some("town_housing"), "Town housing"),
        "farm" => (Some("farmland"), "Farmland"),
        "pasture" => (Some("pasture"), "Pasture"),
        "forestry" => (Some("woodland"), "Woodland"),
        "wild" => (Some("wild_land"), "Wild land"),
        "quarry" => (None, "Quarry"),
        "claypit" => (None, "Clay pit"),
        "ironmine" => (None, "Iron mine"),
        "coalmine" => (None, "Colliery"),
        "industry" => (None, "Firms"),
        "derelict" => (None, "Derelict ground"),
        other => (None, other),
    };
    match key {
        Some(k) => vec![Piece::Link(Link::Term(k.to_string()), word.to_string())],
        None => plain(word),
    }
}

// ------------------------------------------------------------------ tooltips of things

/// A method's tooltip, from its recipe in the record: what it makes, from what, per unit.
pub fn method_tip(r: &Replay, m: usize, fam: Option<&FamilyView>) -> Option<TipBody> {
    let g = words::glossary();
    let me = r.header.methods.get(m)?;
    let good = good_label(r, me.out).to_lowercase();
    let mut from: Vec<String> = me.inputs.iter().flatten().map(|&(gd, _)| good_label(r, gd).to_lowercase()).collect();
    if me.heat.is_some_and(|h| h > 0.0) {
        from.push("heat".into());
    }
    if me.labor.is_some_and(|l| l > 0.0) {
        from.push("labour".into());
    }
    let list = match from.len() {
        0 => String::new(),
        1 => format!(" from {}", from[0]),
        n => format!(" from {} and {}", from[..n - 1].join(", "), from[n - 1]),
    };
    let says = g.pieces(&format!("A method that makes {good}{list}."), None);
    let mut rows = Vec::new();
    for &(gd, a) in me.inputs.iter().flatten() {
        rows.push((vec![Piece::Link(Link::Good(gd), good_label(r, gd))], plain(qty(a as f64))));
    }
    if let Some(h) = me.heat.filter(|&h| h > 0.0) {
        rows.push((vec![Piece::Link(Link::Term("heat".into()), "Heat".into())], plain(qty(h as f64))));
    }
    if let Some(l) = me.labor.filter(|&l| l > 0.0) {
        rows.push((vec![Piece::Link(Link::Term("labour".into()), "Labour".into())], plain(format!("{} mo", qty(l as f64)))));
    }
    rows.push(row("Standard size", format!("{} {good}/mo", qty(me.k0 as f64))));
    if let Some(t) = fam.and_then(|f| f.method_group.get(m).copied().flatten()) {
        rows.push(row("Trade", t.label()));
    }
    let links = ["method", "technology"].iter().filter_map(|k| g.get(k).map(|t| (Link::Term(k.to_string()), t.name.clone()))).collect();
    Some(TipBody { term: me.label.clone(), value: Some(format!("Per unit of {good}")), says, rows, links, small: false })
}

/// A town's small tooltip, from the record: its people that year, and when it was founded.
pub fn town_tip(r: &Replay, t: usize, k: usize) -> Option<TipBody> {
    let town = r.header.towns.get(t)?;
    let i = k * r.n_towns + t;
    let people = |p: f32| format!("{} people", thousands(p.max(0.0).round() as u64));
    let mut rows = Vec::new();
    if let (Some(&area), Some(&urban)) = (r.town_pop.get(i), r.town_urban.get(i)) {
        rows.push((vec![Piece::Link(Link::Term("market_area".into()), "Market area".into())], plain(people(area))));
        rows.push(row("Town", people(urban)));
    }
    rows.push(row("Founded", if town.founded < r.y0 { format!("before {}", r.y0) } else { town.founded.to_string() }));
    Some(TipBody { term: town.name.clone(), value: Some((r.y0 + k as i32).to_string()), rows, small: true, ..default() })
}

/// A good's small tooltip, from the record: its lowest and highest price among the towns that year.
pub fn good_tip(r: &Replay, g: usize, k: usize) -> Option<TipBody> {
    let label = r.header.goods.get(g)?.label.clone();
    let ng = r.header.goods.len();
    let mut seen: Vec<(f32, usize)> = (0..r.n_towns).filter_map(|t| r.town_price.get((k * r.n_towns + t) * ng + g).copied().filter(|p| p.is_finite() && *p > 0.0).map(|p| (p, t))).collect();
    seen.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut rows = Vec::new();
    let name = |t: usize| r.header.towns.get(t).map_or(String::new(), |x| x.name.clone());
    if let (Some(lo), Some(hi)) = (seen.first(), seen.last()) {
        rows.push(row("Lowest price", format!("{} \u{b7} {}", money(lo.0 as f64), name(lo.1))));
        rows.push(row("Highest price", format!("{} \u{b7} {}", money(hi.0 as f64), name(hi.1))));
    }
    Some(TipBody { term: label, value: Some((r.y0 + k as i32).to_string()), rows, small: true, ..default() })
}

// ------------------------------------------------------------------ the plan

fn line(g: &Glossary, section: &str, label: Vec<Piece>, detail: Option<String>, value: Vec<Piece>, key: &str, rows: Vec<(Vec<Piece>, Vec<Piece>)>) -> Line {
    let v: String = value.iter().map(|p| p.text()).collect();
    let tip = TipBody::term(g, key, Some(v), rows);
    let id = format!("{section}/{}", label.iter().map(|p| p.text()).collect::<String>());
    Line { label, detail, value, tip, id }
}

fn term_label(g: &Glossary, key: &str) -> Vec<Piece> {
    plain(g.get(key).map_or(key.to_string(), |t| t.name.clone()))
}

/// The card of firm `f` in year `k`, with the simulation's answer about it if
/// there is one, and the map view's line.
pub fn plan(r: &Replay, f: usize, k: usize, fam: Option<&FamilyView>, told: Option<&Told>, foot: Option<String>) -> Option<Plan> {
    let g = words::glossary();
    let firm = r.header.firms.get(f)?;
    let m = r.header.methods.get(firm.method)?;
    let good = good_label(r, m.out);
    let good_l = good.to_lowercase();
    let town = usize::try_from(firm.town).ok().filter(|&t| t < r.header.towns.len());
    let year = if firm.born < r.y0 { format!("before {}", r.y0) } else { firm.born.to_string() };
    let mut top = vec![Piece::Link(Link::Method(firm.method), m.label.clone())];
    if let Some(t) = town {
        top.push(Piece::Text(" \u{b7} ".into()));
        top.push(Piece::Link(Link::Town(t), r.header.towns[t].name.clone()));
    }
    top.push(Piece::Text(format!(" \u{b7} {year}")));

    let mut sections = Vec::new();
    // capacity and size, and the return the founders expected
    let kk = replay::firm_k(r, f, k) as f64;
    let size = kk / (m.k0 as f64).max(1e-6);
    let cap = format!("{} {good_l}/mo", qty(kk));
    let mut first = vec![line(
        g,
        "",
        term_label(g, "capacity"),
        Some(times(size)),
        plain(cap),
        "capacity",
        vec![
            (vec![Piece::Link(Link::Term("size".into()), "Size".into())], plain(times(size))),
            row("Standard size", format!("{} {good_l}/mo", qty(m.k0 as f64))),
            row("Year", (r.y0 + k as i32).to_string()),
        ],
    )];
    let mut rest = Vec::new();
    if let Some(rs) = &firm.reason {
        let num = |key: &str| rs.get(key).and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
        let kind = rs.get("kind").and_then(|v| v.as_str()).unwrap_or("new");
        let built = num("year") as i64;
        let (roi, margin, pout, rent, k_at) = (num("roi"), num("margin"), num("pout"), num("rent"), num("K"));
        // the start, as each of the three kinds told it
        let mut rrows = match kind {
            "old" => vec![row("Start", format!("At work in {}", r.y0)), row("Priced as", "Built new")],
            "expansion" => vec![row("Start", format!("Second site, {built}")), row("Opened", "First site full")],
            _ => {
                let funds = rs.get("town").and_then(|v| v.as_u64()).map(|t| t as usize).filter(|&t| t < r.header.towns.len());
                let mut v = vec![row("Start", format!("Built {built}"))];
                if let Some(t) = funds {
                    v.push((plain("Funds"), vec![Piece::Link(Link::Town(t), r.header.towns[t].name.clone())]));
                }
                v.push(row("Sites priced", thousands(num("n_cand").max(0.0) as u64)));
                v
            }
        };
        // the sums behind it: a year of margin at 85% of capacity, less a year's ground rent, over the cost to build
        if k_at.is_finite() && roi.is_finite() && roi.abs() > 1e-6 {
            let year_margin = 12.0 * k_at * 0.85 * margin;
            rrows.push((vec![Piece::Link(Link::Term("margin".into()), "Margin".into()), Piece::Text("/yr".into())], plain(money(year_margin))));
            rrows.push((vec![Piece::Link(Link::Term("ground_rent".into()), "Ground rent".into()), Piece::Text("/yr".into())], plain(money(-12.0 * rent))));
            rrows.push(row("Cost to build", money((year_margin - 12.0 * rent) / roi)));
        }
        first.push(line(g, "", term_label(g, "return"), None, plain(format!("{:.0}%/yr", roi * 100.0)), "return", rrows));

        // a unit's sums: the price, each cost as an amount times a price, the margin
        let per = format!("Per {good_l}");
        let mut costs: Vec<(Vec<Piece>, String, f64, &str)> = Vec::new();
        if let Some(ins) = rs.get("inputs").and_then(|v| v.as_array()) {
            for q in ins {
                let gd = q.get(0).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let a = q.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0);
                let p = q.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0);
                costs.push((vec![Piece::Link(Link::Good(gd), good_label(r, gd))], format!("{} \u{d7} {}", qty(a), money(p)), a * p, "materials"));
            }
        }
        if let Some(h) = rs.get("heat").and_then(|v| v.as_array()) {
            let (a, p) = (h.first().and_then(|v| v.as_f64()).unwrap_or(0.0), h.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0));
            if a > 0.0 {
                costs.push((term_label(g, "heat"), format!("{} \u{d7} {}", qty(a), money(p)), a * p, "heat"));
            }
        }
        if let Some(l) = rs.get("labor").and_then(|v| v.as_array()) {
            let (a, w) = (l.first().and_then(|v| v.as_f64()).unwrap_or(0.0), l.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0));
            if a > 0.0 {
                costs.push((term_label(g, "labour"), format!("{} mo \u{d7} {}", qty(a), money(w)), a * w, "labour"));
            }
        }
        let pence = pence_that_add_up(pout, margin, &costs.iter().map(|c| c.2).collect::<Vec<_>>());
        let mut lines = vec![line(g, &per, term_label(g, "price"), None, plain(money(pout)), "price", town.map(|t| vec![(plain("Market town"), vec![Piece::Link(Link::Town(t), r.header.towns[t].name.clone())])]).unwrap_or_default())];
        let mut margin_rows = vec![row("Price", money(pout))];
        for ((label, detail, _, key), p) in costs.iter().zip(&pence) {
            let (amount, price) = detail.split_once(" \u{d7} ").unwrap_or((detail.as_str(), ""));
            let parts = vec![row("Amount", amount), row("Price", price), row("Cost", pence_money(*p))];
            lines.push(line(g, &per, label.clone(), Some(detail.clone()), plain(pence_money(-*p)), key, parts));
            margin_rows.push((label.clone(), plain(pence_money(-*p))));
        }
        lines.push(line(g, &per, term_label(g, "margin"), None, plain(money(margin)), "margin", margin_rows));
        sections.push(Section { heading: None, lines: std::mem::take(&mut first) });
        sections.push(Section { heading: Some(per), lines });

        // the ground: the rent bid and the use it outbid
        let displaced = rs.get("displaced").and_then(|v| v.as_str()).unwrap_or("wild");
        let bid_year = if kind == "old" { r.y0 as i64 } else { built };
        rest.push(line(g, "", term_label(g, "ground_rent"), None, plain(format!("{}/mo", money(rent))), "ground_rent", vec![row(format!("Bid in {bid_year}"), format!("{}/mo", money(rent))), (plain("Outbid"), outbid(displaced))]));
    } else {
        sections.push(Section { heading: None, lines: std::mem::take(&mut first) });
    }
    // the owner: as the simulation says, else as the family's books say, else the town's savers
    let live_owner = told.and_then(|t| t.groups.first()).and_then(|gr| gr.rows.iter().find(|r| r.0 == "Owner")).map(|r| r.1.clone());
    let owner = live_owner.or_else(|| firm.owner.and_then(|o| fam.and_then(|v| v.families.get(o)).map(|x| format!("{} family", x.name)))).or_else(|| town.map(|t| format!("Savers of {}", r.header.towns[t].name)));
    if let Some(o) = owner {
        rest.push(line(g, "", term_label(g, "owner"), None, plain(o), "owner", Vec::new()));
    }
    if !rest.is_empty() {
        sections.push(Section { heading: None, lines: rest });
    }
    // the simulation's rows, under the months they are of
    if let Some(t) = told {
        for gr in &t.groups {
            let lines: Vec<Line> = gr
                .rows
                .iter()
                .filter(|(n, _)| n != "Owner" && n != "Market town")
                .map(|(n, v)| {
                    let key = g.key_of(n).unwrap_or(n).to_string();
                    let parts = if key == "site" { vec![row("Measured from", "North-west corner")] } else { Vec::new() };
                    line(g, &gr.name, plain(n.clone()), None, plain(v.clone()), &key, parts)
                })
                .collect();
            if !lines.is_empty() {
                sections.push(Section { heading: Some(gr.name.clone()), lines });
            }
        }
    }
    Some(Plan { top, sections, foot })
}

// ------------------------------------------------------------------ drawing it

pub fn plugin(app: &mut App) {
    app.add_systems(Update, draw.after(crate::inspect_panel));
}

#[allow(clippy::too_many_arguments)]
fn draw(
    mut commands: Commands,
    font: Res<UiFont>,
    picked: Res<Picked>,
    tl: Res<Timeline>,
    data: Res<Data>,
    ov: Res<crate::overlay::MapOverlay>,
    months: Res<crate::live::Months>,
    fam: Option<Res<FamilyView>>,
    answers: Option<Res<crate::inspector::Answers>>,
    sandbox: Option<Res<crate::sandbox::Sandbox>>,
    live: Option<Res<crate::live::Live>>,
    q: Query<(Entity, Option<&Children>), With<FirmCard>>,
    mut drawn: Local<Option<Plan>>,
    mut seen: Local<Option<(Option<Pick>, usize, u64, Option<crate::overlay::Overlay>, usize, Option<usize>, bool)>>,
) {
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    // nothing it is made from has changed: the plan is not made again
    let key = (picked.what, k, answers.as_ref().map_or(0, |a| a.version), ov.kind, ov.kind.map_or(0, |kind| ov.which(kind)), ov.month(), live.as_ref().is_some_and(|l| crate::inspector::at_present(&tl, &data, l)));
    if *seen == Some(key) && !fam.as_ref().is_some_and(|f| f.is_changed()) {
        return;
    }
    *seen = Some(key);
    let plan = match picked.what {
        Some(Pick::Firm(f)) => {
            // the simulation's rows, in a sandbox at the present (inspector.rs asks for them)
            let present = live.as_ref().is_some_and(|l| crate::inspector::at_present(&tl, &data, l));
            let told = if sandbox.is_some() && present { answers.as_ref().and_then(|a| a.get(Thing::Works(f))).and_then(|t| t.as_ref().ok()) } else { None };
            let foot = ov.said(&data.r, &months, k, data.r.header.firms[f].cell);
            plan(&data.r, f, k, fam.as_deref(), told, foot)
        }
        _ => None,
    };
    if *drawn == plan {
        return;
    }
    let Ok((e, children)) = q.single() else { return };
    if let Some(ch) = children {
        for c in ch.iter() {
            commands.entity(c).despawn();
        }
    }
    if let Some(p) = &plan {
        let f = font.0.clone();
        commands.entity(e).with_children(|c| spawn_plan(c, &f, p));
    }
    *drawn = plan;
}

fn rule(p: &mut ChildSpawnerCommands) {
    p.spawn((Node { height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(5.0)), ..default() }, BackgroundColor(RULE)));
}

/// The sizes of the card's type: its line under the name, its rows, a row's small figure, a heading.
const TOP_SIZE: f32 = 14.0;
const ROW_SIZE: f32 = 14.0;
const DETAIL_SIZE: f32 = 12.0;
const HEAD_SIZE: f32 = 11.0;

fn spawn_plan(p: &mut ChildSpawnerCommands, f: &Handle<Font>, plan: &Plan) {
    tips::spawn_pieces(p, f, &plan.top, TOP_SIZE, MUTED, None, Node::default());
    for s in &plan.sections {
        rule(p);
        if let Some(h) = &s.heading {
            p.spawn((Text::new(h.to_uppercase()), TextFont { font: f.clone().into(), font_size: FontSize::Px(HEAD_SIZE), ..default() }, TextColor(MUTED), Node { margin: UiRect::new(Val::Px(6.0), Val::Px(0.0), Val::Px(2.0), Val::Px(2.0)), ..default() }));
        }
        for l in &s.lines {
            p.spawn((
                Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, column_gap: Val::Px(10.0), padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() },
                BackgroundColor(Color::NONE),
                Tip { body: Arc::new(l.tip.clone()), label: l.label.iter().map(|x| x.text()).collect(), id: l.id.clone() },
                LitRow,
            ))
            .with_children(|r| {
                tips::spawn_pieces(r, f, &l.label, ROW_SIZE, INK, None, Node { flex_shrink: 0.0, ..default() });
                r.spawn(Node { column_gap: Val::Px(10.0), align_items: AlignItems::Baseline, ..default() }).with_children(|v| {
                    if let Some(d) = &l.detail {
                        v.spawn((Text::new(d.clone()), TextFont { font: f.clone().into(), font_size: FontSize::Px(DETAIL_SIZE), ..default() }, TextColor(MUTED)));
                    }
                    tips::spawn_pieces(v, f, &l.value, ROW_SIZE, INK, None, Node::default());
                });
            });
        }
    }
    if let Some(foot) = &plan.foot {
        rule(p);
        p.spawn((Text::new(foot.clone()), TextFont { font: f.clone().into(), font_size: FontSize::Px(13.0), ..default() }, TextColor(MUTED)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A record with one firm, as the core's recorder writes a firm founded with money from a town.
    fn fairfax() -> Replay {
        let mut r = replay::tiny(1750);
        r.header.goods = serde_json::from_value(serde_json::json!([{"name": "cloth", "label": "Cloth"}, {"name": "wool", "label": "Wool"}])).unwrap();
        r.header.methods = serde_json::from_value(serde_json::json!([{"name": "powered_mill", "label": "Powered mill", "out": 0, "K0": 0.12, "inputs": [[1, 1.4]], "heat": 0.5, "labor": 0.5}])).unwrap();
        r.header.firms = serde_json::from_value(serde_json::json!([{
            "name": "Fairfax Mill", "method": 0, "cell": 0, "town": 0, "born": 1750,
            "reason": {"kind": "new", "year": 1750, "town": 0, "cell": 0, "K": 2.5, "roi": 0.2306, "margin": 2.9918, "pout": 10.6137,
                       "inputs": [[1, 1.4213, 2.7891]], "heat": [0.5512, 0.7431], "labor": [0.5087, 6.4120], "rent": 2.3871,
                       "displaced": "urban", "agglo": 1.0, "dist": 1.2, "n_cand": 14}
        }])).unwrap();
        r
    }

    #[test]
    fn the_costs_add_up_to_the_margin_in_pence() {
        // the record's own: price less wool, heat and labour, each from its unrounded amount and price
        let costs = [1.4213 * 2.7891, 0.5512 * 0.7431, 0.5087 * 6.4120];
        let p = pence_that_add_up(10.6137, 2.9918, &costs);
        assert_eq!(1061 - p.iter().sum::<i64>(), 299, "{p:?}");
        // and in a case where each rounded alone would be a penny out
        let p = pence_that_add_up(1.0, 0.0, &[0.335, 0.335, 0.33]);
        assert_eq!(p.iter().sum::<i64>(), 100);
    }

    #[test]
    fn the_firms_card_has_no_sentence_and_every_label_is_a_term() {
        let r = fairfax();
        let g = words::glossary();
        let plan = plan(&r, 0, 0, None, None, None).unwrap();
        let goods: Vec<String> = r.header.goods.iter().map(|x| x.label.clone()).collect();
        for s in &plan.sections {
            if let Some(h) = &s.heading {
                assert!(h.split_whitespace().count() <= 3, "{h}");
            }
            for l in &s.lines {
                let label: String = l.label.iter().map(|p| p.text()).collect();
                // a label is a term of the glossary, or the name of a good (a thing on the island)
                assert!(g.key_of(&label).is_some() || goods.contains(&label), "\"{label}\" is no term of data/words.toml");
                assert_eq!(words::label_fault(&label), None);
                let value: String = l.value.iter().map(|p| p.text()).collect();
                assert!(!value.contains(". ") && !value.ends_with('.') && value.split_whitespace().count() <= 3, "{label}: {value}");
            }
        }
        let (card, _) = plan.words();
        assert!(card.iter().all(|w| words::kept_out_in(w).is_empty()), "{card:?}");
    }

    #[test]
    fn every_number_of_the_old_card_is_on_the_new_card_or_in_its_tooltips() {
        let r = fairfax();
        let old = crate::describe(&r, &[(0.0, 0.0)], Pick::Firm(0), 0, &[]);
        let plan = plan(&r, 0, 0, None, None, None).unwrap();
        let (card, tipped) = plan.words();
        let new = format!("{} {}", card.join(" | "), tipped.join(" | ")).replace(',', "");
        let numbers = |s: &str| -> Vec<String> {
            s.split(|c: char| !(c.is_ascii_digit() || c == '.')).map(|w| w.trim_matches('.').to_string()).filter(|w| w.chars().any(|c| c.is_ascii_digit())).collect()
        };
        let have: Vec<f64> = numbers(&new).iter().filter_map(|n| n.parse().ok()).collect();
        let mut missing = Vec::new();
        // a number of the old card is on the new one if a figure there, rounded as the old card rounded it, reads the same
        for n in numbers(&old.replace(',', "")) {
            let places = n.split_once('.').map_or(0, |(_, d)| d.len());
            if !have.iter().any(|&m| format!("{m:.places$}") == n) {
                missing.push(n);
            }
        }
        assert!(missing.is_empty(), "numbers of the old card not on the new: {missing:?}\nold: {old}\nnew: {new}");
        // and the words of the card are counted, for the notes
        let words: usize = card.iter().map(|w| w.split_whitespace().filter(|x| x.chars().any(char::is_alphabetic)).count()).sum();
        println!("old card: {} words; new card: {words} words on it", old.split_whitespace().filter(|x| x.chars().any(char::is_alphabetic)).count());
    }
}
