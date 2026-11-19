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

use crate::accounts;
use crate::econ;
use crate::families;
use crate::sim::Sim;
use crate::util::{fmt_g6, fmt_pct, fmt_thousands_f0};

/// A thing that can be asked about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Thing {
    /// A town, by its slot (`State::town_name`).
    Town(usize),
    /// A works, by its slot (`State::f_name`).
    Works(usize),
    /// A square of land, by its index (`y * W + x`).
    Cell(usize),
    Island,
}

/// What the simulation knows of a thing: a title, the month it stands at, and
/// groups of rows.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Told {
    pub title: String,
    /// The month about to be played, as "March 1821".
    pub as_of: String,
    pub groups: Vec<Group>,
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
    let (title, groups) = match thing {
        Thing::Town(t) => town(sim, t)?,
        Thing::Works(f) => works(sim, f)?,
        Thing::Cell(c) => cell(sim, c)?,
        Thing::Island => island(sim),
    };
    Ok(Told { title, as_of: as_of(sim), groups })
}

/// The month about to be played, from the months played so far.
pub fn as_of(sim: &Sim) -> String {
    format!("{} {}", MONTHS[sim.t % 12], sim.content.start_year + (sim.t / 12) as i64)
}

// ---------------------------------------------------------------- figures in words

/// A count or a large sum of money, in whole units with thousands marked.
fn whole(x: f64) -> String {
    fmt_thousands_f0(x)
}

/// A price, a wage or a rent: whole units from a hundred up, two places from
/// one, three below it, so that a price of grain near one reads 0.581.
fn amount(x: f64) -> String {
    if x.abs() >= 100.0 {
        fmt_thousands_f0(x)
    } else if x.abs() >= 1.0 {
        format!("{x:.2}")
    } else {
        format!("{x:.3}")
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
fn and_list(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        n => format!("{} and {}", items[..n - 1].join(", "), items[n - 1]),
    }
}

fn lower(s: &str) -> String {
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
    g.row("plant", format!("makes up to {} {good} a month", whole(st.f_K[f])));
    g.row("plant on order", format!("{} a month more", whole(st.f_order_K[f])));
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
        g.row(format!("made in {last}"), format!("{} {good}, worth {}", whole(y.out), whole(y.value)));
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
            g.row("the way to market", format!("{}, as many squares of open lowland", amount(d)));
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
    ("The island".to_string(), groups)
}
