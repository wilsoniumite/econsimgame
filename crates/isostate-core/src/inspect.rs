//! What the simulation knows of one thing: a town, a works, a square of land or
//! the island, as rows of a name and a figure in words (`docs/designs/sandbox.md`,
//! sections 4.5 and 6.2). It is what the viewer's inspector shows under a clicked
//! thing, and the viewer draws it without understanding it: every figure is made
//! into words here, with the core's own `util::fmt_thousands_f0` and `fmt_pct`,
//! so that no economics is typed twice.
//!
//! It only reads. Nothing here is stored, drawn or changed, so a simulation that
//! is told about a thousand times runs exactly as one that is never asked.
//!
//! The figures are the state's as it stands before the month about to be played
//! (`Told::as_of`). The year's trade and a works' last year are the books' of
//! the last year they closed (`accounts`), and are left out while the books are
//! off. Money is in the island's units, quantities in each good's units.
//!
//! A thing is asked for by the number the simulation keeps it under, which is
//! also the number the record gives it: a town by its slot, a works by its slot
//! in the record's list of works (`record.rs`, the header's `firms`, one entry
//! for every works that has ever stood, in order), so the viewer asks
//! `Thing::Works(f)` for its `Pick::Firm(f)`. A square is `y * W + x`. `Thing`
//! is `Copy`, `Eq` and `Hash`, so a card can be kept by what it tells of, and
//! both `Thing` and `Told` are `Send`, so the question goes to the simulation's
//! thread and the card comes back (`tests/inspect.rs` holds both to that).
//!
//! A town's card also carries its ideas (`Told::ideas`, `town_ideas`): every
//! idea of the data the run plays, the tree's methods and techniques and the
//! uses of the tasks rule, each known here, known elsewhere on the island, or
//! found nowhere yet, in groups as the tree groups them, each with a sentence
//! on what it opened, where it is known or what it waits on, and whether the
//! hand's edit for an idea would find it here. The state does not keep when
//! each town came to know an idea, so whoever wants the years watches the
//! island at each year's end (`Learnt`) and asks with what it has watched
//! (`tell_with`); the viewer's thread does. A card asked without it dates only
//! what the state holds: where and when each idea was found.

use crate::accounts;
use crate::econ;
use crate::edits;
use crate::families;
use crate::sim::Sim;
use crate::util::{fmt_g6, fmt_pct, fmt_thousands_f0};

/// A thing that can be asked about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Thing {
    /// A town, by its slot (`State::town_name`).
    Town(usize),
    /// A works, by its slot (`State::f_name`), which is its place in the
    /// record's list of works.
    Works(usize),
    /// A square of land, by its index (`y * W + x`).
    Cell(usize),
    Island,
    /// MAKING (making.rs): a town's making in the last year its books
    /// closed, by the town's slot, for the panel that draws it as a chart.
    /// Asked for only while the panel is open.
    Making(usize),
}

/// What the simulation knows of a thing: a title, the month it stands at, and
/// groups of rows.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Told {
    pub title: String,
    /// The month about to be played, as "March 1821".
    pub as_of: String,
    pub groups: Vec<Group>,
    /// A town's ideas, every one of the data's in one of three states
    /// (`town_ideas`); None for anything that is not a town.
    pub ideas: Option<Ideas>,
    /// MAKING (making.rs): a town's making, for `Thing::Making` alone.
    pub making: Option<crate::making::Making>,
}

/// A group of rows under a heading: each row a name and a figure in words.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Group {
    pub name: String,
    pub rows: Vec<(String, String)>,
}

impl Group {
    fn new(name: impl Into<String>) -> Group {
        Group { name: name.into(), rows: Vec::new() }
    }

    fn row(&mut self, name: impl Into<String>, figure: impl Into<String>) {
        self.rows.push((name.into(), figure.into()));
    }
}

pub const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/// Tell what the simulation knows of `thing`. A thing that is not there (a slot
/// with no town, a works that has closed, a square off the map or at sea) is
/// refused in a sentence.
pub fn tell(sim: &Sim, thing: Thing) -> Result<Told, String> {
    tell_with(sim, thing, None)
}

/// `tell`, with what has been watched of the towns' knowledge year by year
/// (`Learnt`), from which a town's card dates each idea it came to know. The
/// viewer's thread keeps one and asks with it; without one the card dates
/// only what the state itself holds (where and when each idea was found).
pub fn tell_with(sim: &Sim, thing: Thing, learnt: Option<&Learnt>) -> Result<Told, String> {
    // MAKING (making.rs): a town's making is a chart, not rows
    if let Thing::Making(t) = thing {
        let m = crate::making::town_making(sim, t)?;
        return Ok(Told { title: m.name.clone(), as_of: as_of(sim), groups: Vec::new(), ideas: None, making: Some(m) });
    }
    let (title, groups) = match thing {
        Thing::Town(t) => town(sim, t)?,
        Thing::Works(f) => works(sim, f)?,
        Thing::Cell(c) => cell(sim, c)?,
        Thing::Island => island(sim),
        Thing::Making(_) => unreachable!("told above"),
    };
    let ideas = match thing {
        Thing::Town(t) => Some(town_ideas(sim, t, learnt)?),
        _ => None,
    };
    Ok(Told { title, as_of: as_of(sim), groups, ideas, making: None })
}

/// The month about to be played, from the months played so far.
pub fn as_of(sim: &Sim) -> String {
    format!("{} {}", MONTHS[sim.t % 12], sim.content.start_year + (sim.t / 12) as i64)
}

// ---------------------------------------------------------------- figures in words

/// A count or a large sum of money, in whole units with thousands marked.
pub(crate) fn whole(x: f64) -> String {
    fmt_thousands_f0(x)
}

/// A price, a wage or a rent: whole units from a hundred up, two places from
/// one, three below it, so that a price of grain near one reads 0.581.
pub(crate) fn amount(x: f64) -> String {
    if x.abs() >= 100.0 {
        fmt_thousands_f0(x)
    } else if x.abs() >= 1.0 {
        format!("{x:.2}")
    } else {
        format!("{x:.3}")
    }
}

/// A quantity of a good: whole units from ten up, one place from one, two
/// below, so that a millwright's plant of a third of a unit of machinery a
/// month does not read as nothing (the tree's dearer goods come in small
/// numbers).
pub(crate) fn qty(x: f64) -> String {
    if x.abs() >= 10.0 {
        fmt_thousands_f0(x)
    } else if x == 0.0 {
        "0".to_string()
    } else if x.abs() >= 1.0 {
        format!("{x:.1}")
    } else {
        format!("{x:.2}")
    }
}

fn pct(x: f64) -> String {
    fmt_pct(x, 0)
}

/// A year that may be before the run began.
fn since(sim: &Sim, year: i64) -> String {
    if year < sim.content.start_year { format!("before {}", sim.content.start_year) } else { year.to_string() }
}

fn months(n: usize) -> String {
    if n == 1 { "1 month".to_string() } else { format!("{n} months") }
}

/// A list of words as a sentence would put it: "a", "a and b", "a, b and c".
pub(crate) fn and_list(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        n => format!("{} and {}", items[..n - 1].join(", "), items[n - 1]),
    }
}

pub(crate) fn lower(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Who holds a family's share, by the family's name.
fn family_name(sim: &Sim, k: usize) -> String {
    match sim.st.families.get(k) {
        Some(f) => format!("the {} family", f.surname),
        None => format!("family {k}"),
    }
}

// ---------------------------------------------------------------- a town

fn town(sim: &Sim, t: usize) -> Result<(String, Vec<Group>), String> {
    let (st, C, P) = (&sim.st, &sim.content, &sim.params);
    if t >= st.town_name.len() || t >= st.T_max {
        return Err(format!("There is no town numbered {t}: the island has had {}.", st.town_name.len()));
    }
    if !st.town_active[t] {
        return Err(format!("{} is no longer a market town.", st.town_name[t]));
    }
    let mut groups = Vec::new();

    let mut g = Group::new("People");
    g.row("a market town since", since(sim, st.town_founded[t]));
    g.row("people in its market area", whole(st.pop[t]));
    g.row("people in the town", whole(st.urban_pop[t]));
    g.row("hands on the land", whole(st.ag_labor[t]));
    g.row("hands in works", whole(st.ind_labor[t]));
    // state (docs/rules/state.md, "The university"): the island's seat, and its university
    let u = &st.purse.uni;
    if u.seat == Some(t) {
        g.row("the island's seat", format!("since {}", u.seat_year));
    }
    if let Some(v) = u.unis.iter().rev().find(|v| v.town == t) {
        let said = match v.closed {
            None => format!("founded {}, {} students", v.founded, whole(v.students())),
            Some(y) => format!("founded {}, closed {y}", v.founded),
        };
        g.row(format!("the University of {}", st.town_name[t]), said);
    }
    groups.push(g);

    let mut g = Group::new("Money and prices");
    g.row("the wage", format!("{} a month", amount(st.wage[t])));
    g.row("the households' money", whole(st.hh_cash[t]));
    g.row("the landlords' money", whole(st.lord_cash[t]));
    g.row("the merchants' money", whole(st.merch_cash[t]));
    if let Some(&grain) = C.good_index.get("grain") {
        g.row("the price of grain", format!("{} a unit", amount(st.price[(t, grain)])));
    }
    groups.push(g);

    if P.harvest.enabled && st.harvest_town_now.len() > t {
        let mut g = Group::new("This year's weather");
        g.row("the harvest on its farms", format!("{} of an average year", pct(st.harvest_town_now[t])));
        g.row("the island's harvest", format!("{} of an average year", pct(st.harvest_now)));
        if let Some(&last) = st.harvest_town.get(t) {
            g.row("its harvest last year", format!("{} of an average year", pct(last)));
        }
        groups.push(g);
    }

    // what the town knows: a method or technique is known once half of it is
    // (the share the record draws as known), and arriving from a fiftieth, the
    // share at which a works may be built on it (`firms`, `families`)
    let mut g = Group::new("What it knows");
    let known = (0..C.NT).filter(|&k| st.know[(t, k)] >= 0.5).count();
    g.row("methods and techniques known", format!("{known} of {}", C.NT));
    let mut arriving: Vec<(f64, usize)> = (0..C.NT).map(|k| (st.know[(t, k)], k)).filter(|&(v, _)| v > 0.02 && v < 0.5).collect();
    arriving.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    if arriving.is_empty() {
        g.row("arriving", "none");
    }
    for (v, k) in arriving {
        g.row(C.tech_label[k].clone(), format!("arriving, {} known", pct(v)));
    }
    groups.push(g);

    // its works, the largest by the plant they stand on at today's prices of
    // capital goods (output is in each good's units, so it cannot rank them)
    let mut g = Group::new("Works");
    let mut here: Vec<(f64, usize)> = (0..st.F_count).filter(|&f| st.f_alive[f] && st.f_town[f] == t).map(|f| (families::works_value(st, C, f), f)).collect();
    here.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    g.row("standing", whole(here.len() as f64));
    for &(v, f) in here.iter().take(3) {
        g.row(st.f_name[f].clone(), format!("{}, plant worth {}", C.method_label[st.f_method[f]], whole(v)));
    }
    groups.push(g);

    if let Some(g) = trade(sim, t) {
        groups.push(g);
    }
    Ok((st.town_name[t].clone(), groups))
}

/// The three lines of the year's trade (`docs/probes/dx-inspector.md`, "At a
/// glance"), from the books of the last year they closed, summed as
/// `examples/flows.rs` sums them. None while the books are off or have no year.
fn trade(sim: &Sim, t: usize) -> Option<Group> {
    let (st, C) = (&sim.st, &sim.content);
    let tb = accounts::town_books(st, C, t, None)?;
    let mut g = Group::new(format!("The year's trade, {}", tb.year));
    // made: the land's and the works' value at the town's prices, as "all of it" in flows
    let made: f64 = tb.land.iter().map(|l| l.value).sum::<f64>() + tb.methods.iter().map(|m| m.value).sum::<f64>();
    let mut makes: Vec<(f64, usize)> = tb.goods.iter().map(|b| (b.made * b.price, b.good)).filter(|x| x.0 > 0.0).collect();
    makes.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut chief: Vec<String> = makes.iter().take(3).map(|&(_, k)| lower(&C.good_label[k])).collect();
    if makes.len() > 3 {
        let more = makes.len() - 3;
        chief.push(if more == 1 { "1 more good".to_string() } else { format!("{more} more goods") });
    }
    g.row("made here", if chief.is_empty() { whole(made) } else { format!("{}: {}", whole(made), and_list(&chief)) });
    // the books keep every flow above nothing; a twentieth of a unit and less
    // is counted in the sums but not as a partner
    let partners = |fl: &[accounts::TownFlow]| {
        let mut p: Vec<usize> = fl.iter().filter(|f| f.qty >= 0.05).map(|f| f.town).collect();
        p.sort_unstable();
        p.dedup();
        p.len()
    };
    let towns = |n: usize| if n == 1 { "1 town".to_string() } else { format!("{n} towns") };
    let (inv, inc) = (tb.received.iter().map(|f| f.value).sum::<f64>(), tb.received.iter().map(|f| f.carting).sum::<f64>());
    g.row("came in, worth where it left", format!("{}: from {}; {} paid to carry it", whole(inv), towns(partners(&tb.received)), whole(inc)));
    let (outv, outc) = (tb.sent.iter().map(|f| f.value).sum::<f64>(), tb.sent.iter().map(|f| f.carting).sum::<f64>());
    g.row("went out, at the prices here", format!("{}: to {}; its carters paid {}", whole(outv), towns(partners(&tb.sent)), whole(outc)));
    Some(g)
}

// ---------------------------------------------------------------- a works

fn works(sim: &Sim, f: usize) -> Result<(String, Vec<Group>), String> {
    let (st, C) = (&sim.st, &sim.content);
    if f >= st.F_count {
        return Err(format!("There is no works numbered {f}: {} have stood on the island.", st.F_count));
    }
    if !st.f_alive[f] {
        return Err(format!("{} closed in {}.", st.f_name[f], st.f_died[f]));
    }
    let (m, t, c) = (st.f_method[f], st.f_town[f], st.f_cell[f]);
    let good = lower(&C.good_label[C.method_out[m]]);
    let W = st.world.W;
    let mut groups = Vec::new();

    let mut g = Group::new("Who and what");
    g.row("method", C.method_label[m].clone());
    g.row("makes", good.clone());
    g.row("since", since(sim, st.f_born[f]));
    g.row("owner", match families::owner_of_firm(st, f) {
        Some(k) => family_name(sim, k),
        None => format!("the savers of {}, no family", st.town_name[t]),
    });
    g.row("market town", st.town_name[t].clone());
    g.row("square", format!("{}, {}", c % W, c / W));
    groups.push(g);

    let mut g = Group::new("Plant and hands");
    g.row("plant", format!("makes up to {} {good} a month", qty(st.f_K[f])));
    g.row("plant on order", format!("{} a month more", qty(st.f_order_K[f])));
    g.row("worth, at today's prices", whole(families::works_value(st, C, f)));
    g.row("hands", whole(st.f_labor[f]));
    g.row("how full it runs", format!("{} of its plant", pct(st.f_u[f])));
    g.row("materials to hand", format!("{} of what it asks for", pct(st.f_fill[f])));
    groups.push(g);

    let mut g = Group::new("Money");
    g.row("cash", whole(st.f_cash[f]));
    g.row("ground rent", format!("{} a month", amount(st.f_rent[f])));
    let played = sim.t % 12;
    g.row(
        "profit so far this year",
        if played == 0 { "none yet: no month of the year played".to_string() } else { format!("{} in {}", whole(st.f_profit_acc[f]), months(played)) },
    );
    // last year's, from the books (`accounts::works_books`), when they kept it
    let last = sim.year - 1;
    if let Some(y) = accounts::works_books(st, C, f).and_then(|wb| wb.years.into_iter().find(|y| y.year == last)) {
        g.row(format!("made in {last}"), format!("{} {good}, worth {}", qty(y.out), whole(y.value)));
        g.row(format!("profit in {last}"), whole(y.profit));
        g.row(format!("paid out in {last}"), whole(y.dividends));
    }
    groups.push(g);
    Ok((st.f_name[f].clone(), groups))
}

// ---------------------------------------------------------------- a square

/// The seven things land holds, as the sectors name their fields and as a
/// reader would. Woods are the trees standing now (`State::cover`), which is
/// what the woodland sector reads, as `land.rs` does.
const LAND: [(&str, &str); 7] = [("fertility", "fertility"), ("pasture", "pasture"), ("forest", "woods"), ("coal", "coal"), ("ore", "ore"), ("clay", "clay"), ("rock", "stone")];

/// A figure of the land, and where it stands among the island's squares of land
/// that have any of it at all: poor in their lowest quarter, rich in their
/// highest, fair between; none where it has nothing.
fn land_word(sim: &Sim, field: &str, c: usize) -> String {
    let w = &sim.st.world;
    let v = if field == "forest" { &sim.st.cover[..] } else { w.field(field) };
    let x = v[c];
    if x <= 0.0 {
        return "none".to_string();
    }
    let (mut some, mut below) = (0usize, 0usize);
    for k in 0..v.len() {
        if w.land[k] && v[k] > 0.0 {
            some += 1;
            if v[k] < x {
                below += 1;
            }
        }
    }
    let share = below as f64 / some.max(1) as f64;
    let word = if share < 0.25 {
        "poor"
    } else if share >= 0.75 {
        "rich"
    } else {
        "fair"
    };
    format!("{}, {word}", amount(x))
}

/// The land use as a reader would name it (`land::USE_NAMES`).
const USE_WORDS: [&str; 12] = ["sea", "wild land", "farmland", "pasture", "woodland", "a quarry", "a clay pit", "an iron mine", "a colliery", "works", "town", "derelict ground"];

fn cell(sim: &Sim, c: usize) -> Result<(String, Vec<Group>), String> {
    let (st, C, P) = (&sim.st, &sim.content, &sim.params);
    let (W, N) = (st.world.W, st.N);
    if c >= N {
        return Err(format!("There is no square numbered {c}: the island's map has {N}."));
    }
    let (x, y) = (c % W, c / W);
    if !st.world.land[c] {
        return Err(format!("The square at {x}, {y} is sea."));
    }
    let market = (st.catch[c] >= 0).then_some(st.catch[c] as usize);
    let mut groups = Vec::new();

    let mut g = Group::new("Whose");
    g.row("owner", match (families::owner_of_cell(st, c), market) {
        (Some(k), _) => family_name(sim, k),
        (None, Some(t)) => format!("the landlords of {}, no family", st.town_name[t]),
        (None, None) => "no one".to_string(),
    });
    // a square is priced on the rent it paid, which is kept only with families
    if families::active(st) && st.cell_rent_year.len() == N {
        g.row("its price", format!("{}, at {} years' purchase", whole(families::land_price(st, P, c)), fmt_g6(P.years_purchase)));
    }
    groups.push(g);

    let mut g = Group::new("Market");
    match market {
        Some(t) => {
            g.row("its market town", st.town_name[t].clone());
            // the path's cost is in squares of open lowland with no road; a
            // unit of a good pays `freight` times its bulk on each
            let d = st.dcatch[c];
            g.row("the way to market", if d < 0.05 { "none: the market is here".to_string() } else { format!("as costly as {d:.1} squares of open lowland") });
            if let Some(&grain) = C.good_index.get("grain") {
                g.row("carting grain there", format!("{} a unit", amount(P.freight * C.weight[grain] * d)));
            }
        }
        None => g.row("its market town", "none: no market reaches it"),
    }
    groups.push(g);

    let mut g = Group::new("Land");
    g.row("used for", USE_WORDS.get(st.land_use[c] as usize).copied().unwrap_or("something else"));
    if !st.urban[c] && st.lam.cols == N {
        // the shares of the square each sector works, and wild land (the last row)
        let mut parts: Vec<(f64, String)> = (0..st.lam.rows)
            .map(|s| (st.lam[(s, c)], if s < C.NS { lower(&C.sector_label[s]) } else { "wild".to_string() }))
            .filter(|p| p.0 >= 0.005)
            .collect();
        parts.sort_by(|a, b| b.0.total_cmp(&a.0));
        if !parts.is_empty() {
            g.row("worked as", parts.iter().map(|(v, n)| format!("{n} {}", pct(*v))).collect::<Vec<_>>().join(", "));
        }
    }
    for (field, word) in LAND {
        g.row(word, land_word(sim, field, c));
    }
    // woods (docs/rules/woods.md): how thick its woods stand against what the
    // ground could carry, and whether its market's foresters manage them
    let pot = st.world.forest_pot[c];
    if P.woods.record && pot > 0.0 {
        let managed = match crate::woods::known_at(st, C, P, c) {
            Some(k) if k >= 0.005 => format!(", managed by {} of its foresters", pct(k)),
            _ => String::new(),
        };
        g.row("woods' cover", format!("{} of what the ground could carry{managed}", pct(st.cover[c] / pot)));
    }
    if st.firm_at[c] >= 0 && st.f_alive.get(st.firm_at[c] as usize) == Some(&true) {
        let f = st.firm_at[c] as usize;
        g.row("works on it", format!("{}, {}", st.f_name[f], lower(&C.method_label[st.f_method[f]])));
    }
    groups.push(g);

    let mut g = Group::new("Ways");
    g.row("road", ["none", "a track", "a road", "a turnpike"].get(st.road[c] as usize).copied().unwrap_or("a road"));
    g.row("railway", if st.rail[c] > 0 { "yes" } else { "none" });
    if !st.canal.is_empty() {
        g.row("canal", if st.canal[c] > 0 { "yes" } else { "none" });
    }
    let w = &st.world;
    g.row("water", if w.navigable[c] {
        "a navigable river"
    } else if w.river[c] {
        "a river"
    } else if w.coast[c] {
        "the sea's shore"
    } else if st.water[c] {
        "a shore"
    } else {
        "none"
    });
    groups.push(g);

    let mut g = Group::new("Rent and weather");
    g.row("rent bid for it now", format!("{} a month", amount(st.cell_rent[c])));
    if let Some(&paid) = st.cell_rent_year.get(c) {
        g.row(format!("rent it paid in {}", sim.year - 1), amount(paid));
    }
    // the weather on the sectors that feel it and work some of the square
    if P.harvest.enabled && st.harvest_mult.cols == N && st.lam.cols == N && !st.urban[c] {
        for (s, label) in C.sector_label.iter().enumerate() {
            let k = st.harvest_mult[(s, c)];
            if k != 1.0 && st.lam[(s, c)] >= 0.005 {
                g.row(format!("this year on its {}", lower(label)), format!("{} of an average year", pct(k)));
            }
        }
    }
    groups.push(g);
    Ok((format!("The square at {x}, {y}"), groups))
}

// ---------------------------------------------------------------- the island

fn island(sim: &Sim) -> (String, Vec<Group>) {
    let (st, P) = (&sim.st, &sim.params);
    let towns: Vec<usize> = (0..st.T_max.min(st.town_name.len())).filter(|&t| st.town_active[t]).collect();
    let mut groups = Vec::new();

    let mut g = Group::new("People");
    g.row("people", whole(towns.iter().map(|&t| st.pop[t]).sum()));
    g.row("people in towns", whole(towns.iter().map(|&t| st.urban_pop[t]).sum()));
    g.row("market towns", whole(towns.len() as f64));
    if P.people.enabled {
        g.row("gone overseas", whole(st.pp_emigrants));
    }
    groups.push(g);

    let mut g = Group::new("Works");
    g.row("standing", whole((0..st.F_count).filter(|&f| st.f_alive[f]).count() as f64));
    g.row("ever built", whole(st.F_count as f64));
    groups.push(g);

    let mut g = Group::new("Money");
    g.row("the island's money", whole(econ::money_total(st)));
    g.row(format!("issued since {}", sim.content.start_year), whole(st.issued));
    g.row(format!("at the start, {}", sim.content.start_year), whole(st.money_start));
    groups.push(g);

    if P.harvest.enabled && !st.harvest_mult.data.is_empty() {
        let mut g = Group::new("This year's weather");
        g.row("the island's harvest", format!("{} of an average year", pct(st.harvest_now)));
        groups.push(g);
    }

    groups.push(by_hand(sim));
    ("The island".to_string(), groups)
}

/// What the hand has done to the island (`State::by_hand`, the ledger of
/// `edits.rs`), always there, so that the Hand tab's foot has its rows from the
/// first month: "none yet" until an edit is carried out.
fn by_hand(sim: &Sim) -> Group {
    let hand = &sim.st.by_hand;
    let mut g = Group::new("By hand");
    if hand.count == 0 {
        for name in ["edits carried out", "money made by hand", "people made by hand"] {
            g.row(name, "none yet");
        }
        return g;
    }
    g.row("edits carried out", whole(hand.count as f64));
    // the share is of the island's money as it stands, the made money in it;
    // money unmade by a levy is a share of what is left, given without a sign
    let total = econ::money_total(&sim.st);
    // (a figure the edits have left at nothing, or brought back to it, is none on balance)
    g.row("money made by hand", if signed(hand.money) == "0" {
        NONE_ON_BALANCE.to_string()
    } else if total > 0.0 {
        format!("{}, {} of the island's money", signed(hand.money), pct(hand.money.abs() / total))
    } else {
        signed(hand.money)
    });
    g.row("people made by hand", if signed(hand.people) == "0" { NONE_ON_BALANCE.to_string() } else { signed(hand.people) });
    g
}

/// A figure of the ledger the edits carried out have left at nothing.
const NONE_ON_BALANCE: &str = "none, on balance";

/// A sum with its sign, "+12,400" or "-6,200"; "0" for less than half a unit
/// either way.
fn signed(x: f64) -> String {
    let w = whole(x.abs());
    if w == "0" {
        w
    } else if x > 0.0 {
        format!("+{w}")
    } else {
        format!("-{w}")
    }
}

// ---------------------------------------------------------------- a town's ideas

/// What has been watched of the towns' knowledge: the year each town came to
/// know each idea of the tree (its knowledge above a fiftieth, `KNOWS`, from
/// which a works of it may be built there), as the atlas's third plate
/// watches it. The state keeps only the knowledge as it stands, so a card
/// that dates each idea needs this kept beside the game, and the viewer's
/// thread keeps one (`year_end` after every year, the years played again when
/// a game is gone on with among them). It only reads the state.
#[derive(Clone, Debug, Default)]
pub struct Learnt {
    towns: usize,
    ideas: usize,
    /// `[town * ideas + idea]`: the year, or -1 while not yet known
    knew: Vec<i64>,
}

impl Learnt {
    /// Watch the year just played (`sim.year - 1`).
    pub fn year_end(&mut self, sim: &Sim) {
        let (st, C) = (&sim.st, &sim.content);
        let year = sim.year - 1;
        if self.towns != st.T_max || self.ideas != C.NT || self.knew.len() != st.T_max * C.NT {
            (self.towns, self.ideas) = (st.T_max, C.NT);
            self.knew = vec![-1; st.T_max * C.NT];
        }
        for t in 0..st.T_max.min(st.town_founded.len()) {
            if !st.town_active[t] {
                continue;
            }
            for k in 0..C.NT {
                let i = t * C.NT + k;
                // (a slot taken by a town founded later starts again)
                if self.knew[i] >= 0 && self.knew[i] < st.town_founded[t] {
                    self.knew[i] = -1;
                }
                if self.knew[i] < 0 && st.know[(t, k)] > KNOWS {
                    self.knew[i] = year;
                }
            }
        }
    }

    /// The year town `t` came to know idea `k` of the tree, if it was watched.
    pub fn year(&self, t: usize, k: usize) -> Option<i64> {
        (t < self.towns && k < self.ideas).then(|| self.knew[t * self.ideas + k]).filter(|&y| y >= 0)
    }
}

/// A town knows an idea once its knowledge of it is above this (`firms`,
/// `families`, `edits`: from here a works of it may be built in the town).
pub const KNOWS: f64 = crate::atlas_ideas::KNOWS;

/// Where an idea stands for one town.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stands {
    /// The town knows it (for a use of the tasks rule: the island has found it).
    Here,
    /// Found on the island, or known before its history began, but not known here yet.
    Elsewhere,
    /// Found nowhere yet.
    Nowhere,
}

/// A town's ideas: every idea of the data the run plays (the tree's methods
/// and techniques, and the uses of the tasks rule), each in one of three
/// states, in groups as the tree groups them (the trades of the works, then
/// the techniques, then the uses).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ideas {
    /// The town, by its slot.
    pub town: usize,
    /// The timeline's two ends: the run's first year and the year in progress.
    pub from: i64,
    pub to: i64,
    pub groups: Vec<IdeaGroup>,
}

impl Ideas {
    /// Every idea, group by group.
    pub fn all(&self) -> impl Iterator<Item = &TownIdea> {
        self.groups.iter().flat_map(|g| g.ideas.iter())
    }

    /// How many ideas stand so.
    pub fn count(&self, stands: Stands) -> usize {
        self.all().filter(|i| i.stands == stands).count()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct IdeaGroup {
    pub name: String,
    pub ideas: Vec<TownIdea>,
}

/// One idea as a town's card tells it.
#[derive(Clone, Debug, PartialEq)]
pub struct TownIdea {
    /// As the data names it (`Content::tech_name`, or a use's name).
    pub name: String,
    pub label: String,
    /// "method", "technique" or "use".
    pub kind: &'static str,
    pub stands: Stands,
    /// Here: the year the town came to know it, where that is known.
    /// Elsewhere: the year it was found on the island. Nowhere: none.
    pub year: Option<i64>,
    /// In a few words, how the town came to know it, or where it stands.
    pub how: String,
    /// The share of the town that knows it, 0 to 1 (for a use, 1 once found).
    pub share: f64,
    /// How many of the island's towns know it.
    pub towns: usize,
    /// What a click on it tells: for one known here what it opened and how
    /// many works use it now; for one known elsewhere where it is known; for
    /// one found nowhere what it waits on.
    pub said: String,
    /// The hand's edit for an idea (`edits::Edit::Discover`) would find it in
    /// this town now: a method or technique found nowhere yet, which leaves
    /// out exactly that edit's two refusals of an idea (known before the
    /// history began, found already).
    pub discover: bool,
}

/// What a technique of the tree changes, since it adds no works of its own
/// (the data says it only in its news); None for one not named here.
fn technique_words(name: &str) -> Option<&'static str> {
    Some(match name {
        "steam_pumping" => "steam engines drain the pits, so that the mines can go deeper",
        "wagonway" => "iron wagonways are laid at the pits, so that one horse hauls what four did",
        "railway" => "railways may be laid between the towns, and goods go by rail",
        "improved_husbandry" => "the farms sow turnips and clover in place of fallow and keep more stock through the winter, so that yields climb",
        "farm_machinery" => "machines come to the farms, so that fewer hands bring in the same harvest",
        "breeding" => "bred seed and stock, and sprays against blight, make the same field give more",
        "managed_woodland" => "woods are planted and felled in rotation, so that the woods of the town's market area grow back faster",
        _ => return None,
    })
}

/// A sentence's first letter in capitals.
fn capital(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// A town's ideas (`Ideas`), from the state as it stands and, for the year
/// each was learnt, from `learnt` where the caller keeps one. It only reads.
pub fn town_ideas(sim: &Sim, t: usize, learnt: Option<&Learnt>) -> Result<Ideas, String> {
    let (st, C, P) = (&sim.st, &sim.content, &sim.params);
    if t >= st.town_name.len() || t >= st.T_max {
        return Err(format!("There is no town numbered {t}: the island has had {}.", st.town_name.len()));
    }
    if !st.town_active[t] {
        return Err(format!("{} is no longer a market town.", st.town_name[t]));
    }
    let (start, now) = (C.start_year, sim.year);
    let active: Vec<usize> = (0..st.T_max.min(st.town_name.len())).filter(|&i| st.town_active[i]).collect();
    let founded = st.town_founded[t];
    // how each idea of the tree was found, from the island's news: by hand, by a pin or by a draw
    let mut found_by: Vec<Option<&str>> = vec![None; C.NT];
    for ev in &sim.events {
        if let Some(k) = ev.tech.filter(|&k| k < C.NT) {
            if found_by[k].is_none() && (ev.kind == "hand" || ev.kind == "discovery") {
                found_by[k] = Some(if ev.kind == "hand" {
                    "by hand"
                } else if ev.title.ends_with("pinned to") {
                    "by a pin, in the year it was pinned to"
                } else {
                    "by a draw, on what the island had made"
                });
            }
        }
    }
    let mut tree: Vec<(Option<families::Group>, usize, TownIdea)> = Vec::new();
    for k in 0..C.NT {
        let method = k < C.M;
        let kind = if method { "method" } else { "technique" };
        let share = st.know[(t, k)];
        // the towns that know it, the most knowing first
        let mut who: Vec<(f64, usize)> = active.iter().map(|&i| (st.know[(i, k)], i)).filter(|&(v, _)| v > KNOWS).collect();
        who.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let opens = |here: bool| -> String {
            if !method {
                let what = technique_words(&C.tech_name[k]).map_or(String::new(), |w| format!(": {w}"));
                return if here { format!("It adds no works of its own, and changes how work is done{what}.") } else { format!("Known here, it would change how work is done{what}.") };
            }
            let good = lower(&C.good_label[C.method_out[k]]);
            let trade = families::Group::of_method(C, k).map_or(String::new(), |g| format!(", in {}", lower(g.label())));
            let what = format!("works of the kind {}, which make {good}{trade}", lower(&C.method_label[k]));
            if !here {
                return format!("Known here, it would open {what}.");
            }
            let (here_n, all_n) = (0..st.F_count).filter(|&f| st.f_alive[f] && st.f_method[f] == k).fold((0, 0), |(h, a), f| (h + (st.f_town[f] == t) as usize, a + 1));
            let stand = if here_n == 1 { "1 stands".to_string() } else { format!("{here_n} stand") };
            format!("It opens {what}: {stand} here now, of {all_n} on the island.")
        };
        let found_here = st.tech_town[k] == t as i64 && st.tech_year[k] >= 0;
        let idea = if share > KNOWS {
            let year = learnt
                .and_then(|l| l.year(t, k))
                .or(found_here.then_some(st.tech_year[k]))
                .or((C.tech_known[k] && founded <= start).then_some(start))
                // (watched, and not known at the last year's end: it came in the year in progress)
                .or(learnt.map(|_| now));
            let how = if C.tech_known[k] && founded <= start {
                format!("known before {start}")
            } else if found_here {
                format!("found here in {}, {}", st.tech_year[k], found_by[k].unwrap_or("by a draw, on what the island had made"))
            } else if founded >= start && year.is_some_and(|y| y <= founded) {
                format!("brought by those who founded the town in {founded}, from the town it grew out of")
            } else {
                "learnt from the other towns along the island's trade".to_string()
            };
            let head = if C.tech_known[k] && founded <= start {
                format!("known here since before {start}")
            } else {
                format!("known here{}, {how}", year.map_or(String::new(), |y| format!(" since {y}")))
            };
            let arriving = if share < 0.5 { ", and more is still arriving" } else { "" };
            let said = format!("{}: {head}. {} of the town knows it{arriving}. {}", C.tech_label[k], capital(&pct(share)), opens(true));
            TownIdea { name: C.tech_name[k].clone(), label: C.tech_label[k].clone(), kind, stands: Stands::Here, year, how, share, towns: who.len(), said, discover: false }
        } else if C.tech_known[k] || st.tech_year[k] >= 0 {
            let year = Some(if C.tech_known[k] { start } else { st.tech_year[k] });
            let found = if C.tech_known[k] {
                format!("it was known before {start}")
            } else {
                let place = st.town_name.get(st.tech_town[k].max(0) as usize).cloned().unwrap_or_default();
                format!("it was found in {place} in {}", st.tech_year[k])
            };
            let mut most: Vec<String> = who.iter().take(3).map(|&(_, i)| st.town_name[i].clone()).collect();
            let more = who.len().saturating_sub(3);
            if more > 0 {
                most.push(if more == 1 { "1 more".to_string() } else { format!("{more} more") });
            }
            let where_ = match who.len() {
                0 => "no town holds a fiftieth of it now".to_string(),
                1 => format!("1 of the island's {} towns knows it: {}", active.len(), most[0]),
                n => format!("{n} of the island's {} towns know it: {}", active.len(), and_list(&most)),
            };
            let reached = if share >= 0.005 { format!(" ({} of the town knows it so far)", pct(share)) } else { String::new() };
            let said = format!(
                "{}: {found}, and {where_}. It has not reached here yet{reached}: it comes along the island's trade as any idea does, and cannot be found here by hand, since it has been found already. {}",
                C.tech_label[k],
                opens(false)
            );
            let how = if who.len() == 1 { "known in 1 town".to_string() } else { format!("known in {} towns", who.len()) };
            TownIdea { name: C.tech_name[k].clone(), label: C.tech_label[k].clone(), kind, stands: Stands::Elsewhere, year, how, share, towns: who.len(), said, discover: false }
        } else {
            let said = format!("{}: found nowhere yet. {} {} Discover it here finds it in {} now, by hand.", C.tech_label[k], tree_waits(sim, k), opens(false), st.town_name[t]);
            TownIdea { name: C.tech_name[k].clone(), label: C.tech_label[k].clone(), kind, stands: Stands::Nowhere, year: None, how: "not yet found".to_string(), share, towns: 0, said, discover: true }
        };
        tree.push((if method { families::Group::of_method(C, k) } else { None }, k, idea));
    }

    // known here first, by the year; then known elsewhere, by the year found; then found
    // nowhere, by history's year in the data and then the data's order
    let order = |mut ideas: Vec<(usize, TownIdea)>, target: &dyn Fn(usize) -> i64| -> Vec<TownIdea> {
        let rank = |s: Stands| match s {
            Stands::Here => 0,
            Stands::Elsewhere => 1,
            Stands::Nowhere => 2,
        };
        ideas.sort_by_key(|(k, i)| (rank(i.stands), i.year.unwrap_or_else(|| target(*k)), *k));
        ideas.into_iter().map(|(_, i)| i).collect()
    };
    let target = |k: usize| C.tech_target.get(k).copied().filter(|&y| y > 0).unwrap_or(i64::MAX);
    let mut groups = Vec::new();
    let mut push = |name: &str, ideas: Vec<TownIdea>| {
        if !ideas.is_empty() {
            groups.push(IdeaGroup { name: name.to_string(), ideas });
        }
    };
    let take = |keep: &dyn Fn(Option<families::Group>, usize) -> bool| tree.iter().filter(|x| keep(x.0, x.1)).map(|x| (x.1, x.2.clone())).collect::<Vec<_>>();
    for g in families::Group::ALL {
        push(g.label(), order(take(&|grp, k| k < C.M && grp == Some(g)), &target));
    }
    push("Other works", order(take(&|grp, k| k < C.M && grp.is_none()), &target));
    push("Ways of working", order(take(&|_, k| k >= C.M), &target));

    // the uses of the tasks rule, found by the island as a whole
    let people: f64 = st.pop.iter().sum::<f64>().max(1.0);
    let mut uses = Vec::new();
    for (u, us) in C.tasks.uses.iter().enumerate() {
        let found = st.tasks.found.get(u).copied().unwrap_or(-1);
        let label = edits::use_label(&us.name);
        let class = crate::tasks::CLASSES.get(us.class).copied().unwrap_or("some");
        let mut on: Vec<String> = Vec::new();
        if us.on_works {
            on.push("in the works".to_string());
        }
        let land: Vec<String> = us.on_sector.iter().enumerate().filter(|&(_, &b)| b).filter_map(|(s, _)| C.sector_label.get(s).map(|l| lower(l))).collect();
        if !land.is_empty() {
            on.push(format!("on the land ({})", and_list(&land)));
        }
        // (a use replaced by later ones reaches none of the places it was laid on: the tasks rule's `replaces`)
        let place = if on.is_empty() { ", though later machines have taken every place it was used in".to_string() } else { format!(", {}", on.join(" and ")) };
        let what = format!("Machines that take over tasks of the {class} kind{place}.");
        let idea = if found >= 0 {
            let said = format!("{label}: found on the island in {found}, by the island as a whole, since a use of the tasks rule is not known town by town. {what} How much of the work machines do is on the Markets panel's Work half.");
            TownIdea { name: us.name.clone(), label, kind: "use", stands: Stands::Here, year: Some(found), how: "found by the island as a whole".to_string(), share: 1.0, towns: active.len(), said, discover: false }
        } else {
            let goods: Vec<String> = us
                .discover
                .iter()
                .map(|&(g, x)| {
                    let made = if us.discover_per_head { st.cumprod[g] / people } else { st.cumprod[g] };
                    let each = if us.discover_per_head { " a head" } else { "" };
                    format!("{} made since {start}, {}{each} of the {}{each} it needs", lower(&C.good_label[g]), qty(made), qty(x))
                })
                .collect();
            let mut waits = Vec::new();
            if !goods.is_empty() {
                waits.push(and_list(&goods));
            }
            if now < us.found_after {
                waits.push(format!("the year {}, before which it is not found", us.found_after));
            }
            if !P.tasks.enabled {
                waits.push("the tasks rule, which is off, so that it is never found".to_string());
            } else if let Some(s) = us.switch.as_ref().filter(|s| !P.tasks.switch_on(s)) {
                waits.push(format!("the switch tasks.{s}, which is off, so that it is never found"));
            }
            let said = format!(
                "{label}: found nowhere yet. It waits on {}; then it is found at the next year's turn, with no draw. {what} A use of the tasks rule is found by the island as a whole, so it cannot be found by hand in one town.",
                if waits.is_empty() { "nothing more".to_string() } else { waits.join("; ") }
            );
            TownIdea { name: us.name.clone(), label, kind: "use", stands: Stands::Nowhere, year: None, how: "not yet found".to_string(), share: 0.0, towns: 0, said, discover: false }
        };
        uses.push((u, idea));
    }
    push("Machines for tasks", order(uses, &|u: usize| C.tasks.uses.get(u).and_then(|us| us.target).unwrap_or(i64::MAX)));
    Ok(Ideas { town: t, from: start, to: now, groups })
}

/// What an idea of the tree found nowhere waits on, in words: the goods it is
/// found on, each with what has been made of it since the start against what
/// it needs, and the idea before it where nobody can yet make one of them;
/// the year it is first sought; a pin; the woods' rule; and the chance a
/// draw finds it as things stand, reckoned as `tech::discovery` reckons it.
fn tree_waits(sim: &Sim, k: usize) -> String {
    let (st, C, P) = (&sim.st, &sim.content, &sim.params);
    let (start, now) = (C.start_year, sim.year);
    let mut waits: Vec<String> = Vec::new();
    let sought = crate::woods::sought(C, P, k);
    if !sought {
        waits.push("the woods' rule, which is off: it is sought only with that rule on".to_string());
    }
    if let Some(p) = st.by_hand.pins.iter().find(|p| p.idea == edits::Idea::Tech(k)) {
        waits.push(match p.when {
            edits::PinWhen::In => format!("a pin by hand, which finds it in {}", p.year),
            edits::PinWhen::NotBefore => format!("a pin by hand, which holds it back until {}", p.year),
        });
    }
    let after = C.tech_after.get(k).copied().unwrap_or(0);
    if now < after {
        waits.push(format!("the year {after}, before which it is not sought"));
    }
    let req: Vec<usize> = (0..C.G).filter(|&g| st.disc_X0[(k, g)] > 0.0).collect();
    let mut goods = Vec::new();
    for &g in &req {
        let (made, need) = (st.cumprod[g], st.disc_X0[(k, g)]);
        let mut words = format!("{} made since {start}, {} of the {} it needs", lower(&C.good_label[g]), whole(made), whole(need));
        if made >= need {
            words.push_str(", met");
        } else if !C.sector_good.contains(&g) {
            // the idea before it: no land makes the good, and no works that makes it is known anywhere
            let makers: Vec<usize> = (0..C.M).filter(|&m| C.method_out[m] == g).collect();
            if !makers.is_empty() && !makers.iter().any(|&m| C.tech_known[m] || st.tech_year[m] >= 0) {
                let names: Vec<String> = makers.iter().map(|&m| lower(&C.method_label[m])).collect();
                let which = if makers.len() == 1 { "itself not found yet".to_string() } else { "none of them found yet".to_string() };
                words.push_str(&format!(" (made only by {}, {which})", names.join(" or ")));
            }
        }
        goods.push(words);
    }
    if !goods.is_empty() {
        waits.push(and_list(&goods));
    }
    let mut out = if waits.is_empty() { "Nothing is asked of the island for it: the island never finds it by itself.".to_string() } else { format!("It waits on {}.", waits.join("; ")) };
    if !req.is_empty() && sought && now >= after {
        let p = req.iter().map(|&g| st.cumprod[g] / st.disc_X0[(k, g)]).fold(f64::INFINITY, f64::min).max(0.0);
        let p4 = p.powf(4.0);
        let hz = P.discovery_rate * p4 / (1.0 + p4);
        let met = if p >= 1.0 { ", every amount being met" } else { "" };
        let chance = if hz < 0.0005 { "less than one in a thousand".to_string() } else { fmt_pct(hz, 1) };
        out.push_str(&format!(" As things stand a draw at the year's end finds it with a chance of {chance} a year{met}."));
    }
    out
}
