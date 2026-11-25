//! A town's making, as the viewer's panel draws it (the designer's note of 20
//! November 2026: "the production flowchart view, which could be per town,
//! with scaled flows and boxes"): for one town and the last year its books
//! closed, a box for each kind of works there and for each use of the land
//! round it, for its households and for the world outside the town, and a
//! ribbon for each good that went from one box to another, as wide as its
//! value in the year.
//!
//! It is an answer of the inspector's kind (`inspect::Thing::Making`): it only
//! reads the books of account (`accounts::town_books`), which keep everything
//! it needs for every year of the run, and the town's knowledge. Nothing is
//! stored or changed, so a game whose panel is open runs exactly as one whose
//! panel is shut.
//!
//! What the books keep, and so what the chart can say:
//!
//! - A kind of works is every method in the town that makes the same good
//!   (the saw pit and the steam sawmill are one kind, sawn timber), since they
//!   sell into one market. Its box names its methods, and its words give each
//!   method's recipe and the methods the town knows for that good and leaves
//!   unused.
//! - The town's market pools each good. The books say who made a good and who
//!   bought it, not who bought from whom, so a ribbon of a good from one box to
//!   another is the first box's share of what the town had of the good times
//!   what the second took of it. That is the only split the books allow.
//! - The flow of each good balances: what was made here, what came in from
//!   other towns and what came out of store is what the eight buyers took, what
//!   was sold to other towns and what went into store or spoiled. "Into store
//!   or spoiled" is made and came in, less bought and sent, when that is more
//!   than nothing, and "out of store" its other side: the year's change of the
//!   merchants' stock, what spoiled, what moved with districts that changed
//!   market and what was oversold, in one figure.
//! - The land's purchases (machines and their fuel, fertiliser, the pits'
//!   rails) go to a box on the right, "Bought by farms and pits", and plant for
//!   works being built and the materials of railways, canals and the
//!   university to one box, "Building", so that the chart flows one way, from
//!   the ground to the household.
//! - Every value is at the town's price of the good in the year, so that a
//!   good's ribbons in and out of the pool are measured alike; a ribbon from
//!   other towns also says what it was worth where it left and what was paid to
//!   carry it, and one to them what their merchants paid to carry it.
//!
//! The columns are set from the town's own flows: the land and what came in
//! from outside on the left, each kind of works one column to the right of the
//! deepest kind that supplies it, and the households and the rest of the sinks
//! on the right. A cycle (electricity into the colliery and coal into the
//! power station) is cut at its smallest link, keeping the heavy ones; a link
//! cut runs back, and the panel draws it as a thin loop. Kinds the town knows
//! the idea of and has none of (`Faint`) stand in the column the tree's own
//! depth gives them (`atlas::depths`).

use std::collections::HashMap;

use crate::accounts::{self, TownBooks};
use crate::content::Content;
use crate::inspect::{KNOWS, amount, and_list, lower, qty, whole};
use crate::sim::Sim;
use crate::util::fmt_pct;

/// At most this many boxes of the land and of works by name; past it the
/// smallest by hands are gathered into "Other works" and "Other land".
pub const MOST_KINDS: usize = 11;

/// What a box stands for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// what came in from other towns, at the left edge
    FromTowns,
    /// what came out of the merchants' stocks
    FromStore,
    /// a use of the land (farms, pasture, woods, a pit)
    Land,
    /// the smaller uses of the land, together
    OtherLand,
    /// a kind of works: the methods that make one good
    Works,
    /// the smaller kinds of works, together
    OtherWorks,
    /// the households, the landlords and the families
    Households,
    /// what the farms, woods and pits bought
    LandBuys,
    /// plant for works being built, and the materials of railways, canals and the university
    Building,
    /// what was sold to other towns, at the right edge
    ToTowns,
    /// what went into the merchants' stocks, or spoiled
    ToStore,
}

impl Kind {
    /// A box ribbons only leave.
    pub fn source(self) -> bool {
        matches!(self, Kind::FromTowns | Kind::FromStore | Kind::Land | Kind::OtherLand)
    }

    /// A box ribbons only enter.
    pub fn sink(self) -> bool {
        matches!(self, Kind::Households | Kind::LandBuys | Kind::Building | Kind::ToTowns | Kind::ToStore)
    }

    /// The land and the works, which have hands.
    pub fn makes(self) -> bool {
        matches!(self, Kind::Land | Kind::OtherLand | Kind::Works | Kind::OtherWorks)
    }
}

/// A box of the chart.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub kind: Kind,
    /// its name on the chart
    pub label: String,
    /// its column, 0 at the left
    pub col: usize,
    /// hands at work, a mean over the year (the households' box: its people)
    pub hands: f64,
    /// works standing, a mean over the year (none for the rest)
    pub works: f64,
    /// what its ribbons in and out are worth in the year, at the town's prices
    pub value_in: f64,
    pub value_out: f64,
    /// the good it makes, for a box of the land or a kind of works
    pub good: Option<usize>,
    /// what it says under the pointer, a line each
    pub said: Vec<String>,
}

impl Node {
    /// What flows through it: the larger of what came in and what went out.
    pub fn through(&self) -> f64 {
        self.value_in.max(self.value_out)
    }
}

/// A ribbon: a good from one box to another in the year.
#[derive(Clone, Debug, PartialEq)]
pub struct Ribbon {
    pub from: usize,
    pub to: usize,
    pub good: usize,
    /// in the good's units, a year
    pub qty: f64,
    /// at the town's price in the year
    pub value: f64,
    /// it runs from a column to one no further right (a cycle cut); a box to
    /// itself is neither, and is not drawn
    pub back: bool,
    pub said: Vec<String>,
}

/// A kind of works the town knows the idea of and has none of.
#[derive(Clone, Debug, PartialEq)]
pub struct Faint {
    /// the good it would make
    pub good: usize,
    pub label: String,
    pub col: usize,
    pub said: Vec<String>,
}

/// One good's flow through the town in the year, in its units: what came
/// into the town's pool (`made + received + from_store`) and what left it
/// (`bought + sent + to_store`), which are equal.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GoodFlow {
    pub good: usize,
    pub made: f64,
    pub received: f64,
    pub from_store: f64,
    /// by the eight buyers: households, landlords, families, works' materials
    /// and fuel, the land, building, and railways, canals and the university
    pub bought: f64,
    pub sent: f64,
    pub to_store: f64,
    /// the town's price in the year
    pub price: f64,
}

/// A town's making in one year.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Making {
    pub town: usize,
    pub name: String,
    /// the year the books closed, which the chart is of
    pub year: i64,
    pub cols: usize,
    pub nodes: Vec<Node>,
    pub ribbons: Vec<Ribbon>,
    pub faint: Vec<Faint>,
    /// every good that moved through the town
    pub goods: Vec<GoodFlow>,
    /// each good's name on the chart and its colour as the data gives it ("#d6b85a")
    pub good_label: Vec<String>,
    pub good_colour: Vec<String>,
    /// hands at work in the town's works and on its land, a mean over the year
    pub hands: f64,
    /// a line on what the chart is
    pub said: String,
}

impl Making {
    /// The goods' flows, by good.
    pub fn flow(&self, g: usize) -> Option<&GoodFlow> {
        self.goods.iter().find(|f| f.good == g)
    }
}

/// The making of town `t` in the last year its books closed. Refused in a
/// sentence where there is no such town, or no books to read.
pub fn town_making(sim: &Sim, t: usize) -> Result<Making, String> {
    let (st, C) = (&sim.st, &sim.content);
    if t >= st.town_name.len() || t >= st.T_max {
        return Err(format!("There is no town numbered {t}: the island has had {}.", st.town_name.len()));
    }
    if !st.town_active[t] {
        return Err(format!("{} is no longer a market town.", st.town_name[t]));
    }
    if !st.acct.on {
        return Err(format!("The books of account are off in this game, and the chart of {}'s making is drawn from them.", st.town_name[t]));
    }
    let tb = accounts::town_books(st, C, t, None).ok_or_else(|| format!("No year has closed yet, so {}'s books have nothing to show: the chart comes after the first January.", st.town_name[t]))?;
    let knows = |m: usize| st.know[(t, m)] > KNOWS;
    Ok(from_books(C, &tb, &knows, &st.town_name))
}

/// Where a figure of the books goes on the chart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Key {
    FromTowns,
    FromStore,
    Land(usize),
    OtherLand,
    Works(usize),
    OtherWorks,
    Households,
    LandBuys,
    Building,
    ToTowns,
    ToStore,
}

impl Key {
    fn kind(self) -> Kind {
        match self {
            Key::FromTowns => Kind::FromTowns,
            Key::FromStore => Kind::FromStore,
            Key::Land(_) => Kind::Land,
            Key::OtherLand => Kind::OtherLand,
            Key::Works(_) => Kind::Works,
            Key::OtherWorks => Kind::OtherWorks,
            Key::Households => Kind::Households,
            Key::LandBuys => Kind::LandBuys,
            Key::Building => Kind::Building,
            Key::ToTowns => Kind::ToTowns,
            Key::ToStore => Kind::ToStore,
        }
    }
}

/// The order the boxes stand in, top to bottom, before the panel lays them out.
fn rank(k: Key) -> (usize, usize) {
    match k {
        Key::FromTowns => (0, 0),
        Key::FromStore => (1, 0),
        Key::Land(s) => (2, s),
        Key::OtherLand => (3, 0),
        Key::Works(g) => (4, g),
        Key::OtherWorks => (5, 0),
        Key::Households => (6, 0),
        Key::LandBuys => (7, 0),
        Key::Building => (8, 0),
        Key::ToTowns => (9, 0),
        Key::ToStore => (10, 0),
    }
}

/// A recipe in words: what goes in for one of what comes out, as the data
/// writes it (a works' own learning makes it leaner as it grows).
pub fn recipe(C: &Content, m: usize) -> String {
    let mut parts: Vec<String> = (0..C.G).filter(|&k| C.m_in[(m, k)] > 0.0).map(|k| format!("{} {}", qty(C.m_in[(m, k)]), lower(&C.good_label[k]))).collect();
    if C.m_heat[m] > 0.0 {
        parts.push(format!("fuel for {} of heat", qty(C.m_heat[m])));
    }
    parts.push(format!("{} of a hand", qty(C.m_labor[m])));
    let plant: Vec<String> = (0..C.G).filter(|&k| C.m_cap[(m, k)] > 0.0).map(|k| format!("{} {}", qty(C.m_cap[(m, k)]), lower(&C.good_label[k]))).collect();
    let plant = if plant.is_empty() { String::new() } else { format!(", on plant of {} for each unit a month", and_list(&plant)) };
    format!("for 1 {}: {}{plant}", lower(&C.good_label[C.method_out[m]]), and_list(&parts))
}

/// The largest few of a list of (value, name), as "Kelside 1,200, Glenwick 800 and 3 more".
fn largest(mut v: Vec<(f64, String)>, few: usize) -> String {
    v.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    let n = v.len();
    let mut out: Vec<String> = v.iter().take(few).map(|(x, s)| format!("{s} {}", whole(*x))).collect();
    if n > few {
        out.push(if n - few == 1 { "1 more".to_string() } else { format!("{} more", n - few) });
    }
    and_list(&out)
}

/// A town's making from its books for a year: `knows` says whether the town
/// knows a method, `names` names the towns. It only reads.
pub fn from_books(C: &Content, tb: &TownBooks, knows: &dyn Fn(usize) -> bool, names: &[String]) -> Making {
    let G = C.G;
    let town_name = |o: usize| names.get(o).cloned().unwrap_or_else(|| format!("town {o}"));

    // ---- the candidates for a box of their own, ranked by hands
    // (kinds of works by the good they make; the land by its use)
    let mut by_good: Vec<Vec<usize>> = vec![Vec::new(); G];
    for (i, mb) in tb.methods.iter().enumerate() {
        by_good[mb.good].push(i);
    }
    let mut cands: Vec<(f64, f64, Key)> = Vec::new();
    for lb in &tb.land {
        if lb.out > 0.0 || lb.hands > 0.0 {
            cands.push((lb.hands, lb.value, Key::Land(lb.sector)));
        }
    }
    for g in 0..G {
        if !by_good[g].is_empty() {
            let hands: f64 = by_good[g].iter().map(|&i| tb.methods[i].hands).sum();
            let value: f64 = by_good[g].iter().map(|&i| tb.methods[i].value).sum();
            cands.push((hands, value, Key::Works(g)));
        }
    }
    cands.sort_by(|a, b| b.0.total_cmp(&a.0).then(b.1.total_cmp(&a.1)).then(rank(a.2).cmp(&rank(b.2))));
    // a gathering of one would only rename it
    let keep = if cands.len() <= MOST_KINDS + 1 { cands.len() } else { MOST_KINDS };
    let gathered: Vec<Key> = cands[keep..].iter().map(|c| c.2).collect();
    let land_key = |s: usize| if gathered.contains(&Key::Land(s)) { Key::OtherLand } else { Key::Land(s) };
    let works_key = |g: usize| if gathered.contains(&Key::Works(g)) { Key::OtherWorks } else { Key::Works(g) };

    // ---- each good through the town's pool
    let mut flows: HashMap<(Key, Key, usize), f64> = HashMap::new();
    let mut goods = Vec::new();
    for gb in &tb.goods {
        let g = gb.good;
        let mut src: Vec<(Key, f64)> = Vec::new();
        // made: by the land and the works that make it, in the books' total
        let mut makers: Vec<(Key, f64)> = tb.land.iter().filter(|l| l.good == g && l.out > 0.0).map(|l| (land_key(l.sector), l.out)).collect();
        makers.extend(by_good[g].iter().map(|&i| (works_key(g), tb.methods[i].out.max(0.0))));
        let sum: f64 = makers.iter().map(|m| m.1).sum();
        if gb.made > 0.0 {
            if sum > 0.0 {
                src.extend(makers.iter().map(|&(k, q)| (k, q * gb.made / sum)));
            } else {
                // (timber of woods cleared, with nobody felling)
                let k = tb.land.iter().find(|l| l.good == g).map_or(Key::OtherWorks, |l| land_key(l.sector));
                src.push((k, gb.made));
            }
        }
        src.push((Key::FromTowns, gb.received));
        let net = gb.made + gb.received - gb.bought - gb.sent;
        src.push((Key::FromStore, (-net).max(0.0)));
        let mut sink: Vec<(Key, f64)> = vec![(Key::Households, gb.households + gb.landlords + gb.families)];
        // the works' materials and fuel, shared by what each kind used
        let works = gb.works_inputs + gb.works_fuel;
        let users: Vec<(Key, f64)> = tb.methods.iter().flat_map(|mb| mb.uses.iter().filter(|u| u.good == g).map(move |u| (mb.good, u.used + u.burnt))).map(|(o, q)| (works_key(o), q.max(0.0))).collect();
        let sum: f64 = users.iter().map(|u| u.1).sum();
        if works > 0.0 {
            if sum > 0.0 {
                sink.extend(users.iter().map(|&(k, q)| (k, q * works / sum)));
            } else {
                sink.push((Key::OtherWorks, works));
            }
        }
        sink.push((Key::LandBuys, gb.land));
        sink.push((Key::Building, gb.building + gb.projects));
        sink.push((Key::ToTowns, gb.sent));
        sink.push((Key::ToStore, net.max(0.0)));
        src.retain(|s| s.1 > 0.0);
        sink.retain(|s| s.1 > 0.0);
        let tin: f64 = src.iter().map(|s| s.1).sum();
        let tout: f64 = sink.iter().map(|s| s.1).sum();
        if tin <= 0.0 && tout <= 0.0 {
            continue;
        }
        let total = tin.max(tout);
        for &(a, qa) in &src {
            for &(b, qb) in &sink {
                *flows.entry((a, b, g)).or_default() += qa * qb / total;
            }
        }
        goods.push(GoodFlow {
            good: g,
            made: gb.made.max(0.0),
            received: gb.received,
            from_store: (-net).max(0.0),
            bought: gb.bought,
            sent: gb.sent,
            to_store: net.max(0.0),
            price: gb.price,
        });
    }

    // ---- the boxes, in a steady order
    let mut keys: Vec<Key> = flows.keys().flat_map(|&(a, b, _)| [a, b]).collect();
    keys.sort_by_key(|&k| rank(k));
    keys.dedup();
    let index: HashMap<Key, usize> = keys.iter().enumerate().map(|(i, &k)| (k, i)).collect();
    let price = |g: usize| tb.goods.get(g).map_or(0.0, |b| b.price);
    let mut flows: Vec<((Key, Key, usize), f64)> = flows.into_iter().collect();
    flows.sort_by_key(|&((a, b, g), _)| (rank(a), rank(b), g));
    let mut ribbons: Vec<Ribbon> = flows.iter().map(|&((a, b, g), q)| Ribbon { from: index[&a], to: index[&b], good: g, qty: q, value: q * price(g), back: false, said: Vec::new() }).collect();

    // ---- the columns: the land and the outside at 0; each kind of works one
    // to the right of the deepest kind that supplies it, the links kept
    // heaviest first and one that would close a cycle left out; the sinks
    // in the last column, never before the third
    let n = keys.len();
    let mut link: HashMap<(usize, usize), f64> = HashMap::new();
    for r in &ribbons {
        if r.from != r.to && !keys[r.from].kind().sink() && !keys[r.to].kind().sink() && !keys[r.to].kind().source() {
            *link.entry((r.from, r.to)).or_default() += r.value;
        }
    }
    let mut links: Vec<((usize, usize), f64)> = link.into_iter().collect();
    links.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    let mut kept: Vec<Vec<usize>> = vec![Vec::new(); n];
    let reaches = |kept: &Vec<Vec<usize>>, from: usize, to: usize| {
        let mut seen = vec![false; n];
        let mut stack = vec![from];
        while let Some(x) = stack.pop() {
            if x == to {
                return true;
            }
            if !std::mem::replace(&mut seen[x], true) {
                stack.extend(kept[x].iter().copied());
            }
        }
        false
    };
    for &((a, b), _) in &links {
        if !reaches(&kept, b, a) {
            kept[a].push(b);
        }
    }
    let mut col: Vec<usize> = keys.iter().map(|k| if k.kind().source() { 0 } else { 1 }).collect();
    for _ in 0..n {
        let mut moved = false;
        for a in 0..n {
            for &b in &kept[a] {
                if col[b] < col[a] + 1 {
                    col[b] = col[a] + 1;
                    moved = true;
                }
            }
        }
        if !moved {
            break;
        }
    }
    let last = keys.iter().zip(&col).filter(|(k, _)| !k.kind().sink()).map(|(_, &c)| c + 1).max().unwrap_or(1).max(2);
    for (k, c) in keys.iter().zip(col.iter_mut()) {
        if k.kind().sink() {
            *c = last;
        }
    }
    for r in ribbons.iter_mut() {
        r.back = r.from != r.to && col[r.from] >= col[r.to];
    }

    // ---- the boxes' figures
    let mut value_in = vec![0.0; n];
    let mut value_out = vec![0.0; n];
    for r in &ribbons {
        value_out[r.from] += r.value;
        value_in[r.to] += r.value;
    }
    let fig = |name: &str| tb.figures.iter().find(|(f, _)| f == name).map_or(0.0, |(_, v)| *v);
    let year = tb.year;
    let gl = |g: usize| lower(&C.good_label[g]);
    // each kind's methods, and those the town knows for its good and leaves unused
    let unused = |g: usize| -> Vec<usize> { (0..C.M).filter(|&m| C.method_out[m] == g && knows(m) && !by_good[g].iter().any(|&i| tb.methods[i].method == m)).collect() };
    let works_words = |gs: &[usize], said: &mut Vec<String>| {
        for &g in gs {
            for &i in &by_good[g] {
                let mb = &tb.methods[i];
                said.push(format!("{}: {} standing, {} hands; {}.", C.method_label[mb.method], works_n(mb.works), whole(mb.hands), recipe(C, mb.method)));
            }
        }
    };
    let mut nodes = Vec::with_capacity(n);
    for (i, &k) in keys.iter().enumerate() {
        let mut said = Vec::new();
        let (kind, label, hands, works, good) = match k {
            Key::FromTowns => {
                let v: f64 = tb.received.iter().map(|f| f.value).sum();
                let c: f64 = tb.received.iter().map(|f| f.carting).sum();
                let towns = partners(&tb.received, &town_name);
                said.push(format!("Came in from {} in {year}: worth {} where it left and {} at the prices here; {} was paid to carry it.", n_towns(towns.len()), whole(v), whole(value_out[i]), whole(c)));
                said.push(format!("The most from {}.", largest(towns, 4)));
                (Kind::FromTowns, "From other towns".to_string(), 0.0, 0.0, None)
            }
            Key::ToTowns => {
                let c: f64 = tb.sent.iter().map(|f| f.carting).sum();
                let towns = partners(&tb.sent, &town_name);
                said.push(format!("Sold to {} in {year}: worth {} at the prices here; their merchants paid {} to carry it.", n_towns(towns.len()), whole(value_in[i]), whole(c)));
                said.push(format!("The most to {}.", largest(towns, 4)));
                (Kind::ToTowns, "To other towns".to_string(), 0.0, 0.0, None)
            }
            Key::FromStore => {
                said.push(format!("Taken out of the merchants' stocks in {year}, worth {}: what they held less than at the year's start, with what came over with districts that joined the market and what was sold before it was in store.", whole(value_out[i])));
                (Kind::FromStore, "Out of store".to_string(), 0.0, 0.0, None)
            }
            Key::ToStore => {
                said.push(format!("Added to the merchants' stocks in {year}, or spoiled in them, worth {}.", whole(value_in[i])));
                (Kind::ToStore, "Into store, or spoiled".to_string(), 0.0, 0.0, None)
            }
            Key::Land(s) => {
                let lb = &tb.land[s];
                let label = C.sector_label[s].clone();
                said.push(format!("{label}, which brings in {}: {} hands.", gl(lb.good), whole(lb.hands)));
                said.push(format!("Brought in {} {} in {year}, sold for {} ({} a unit); paid {} in wages and {} in rent.", qty(lb.out), gl(lb.good), whole(lb.value), amount(if lb.out > 0.0 { lb.value / lb.out } else { 0.0 }), whole(lb.wages), whole(lb.rent)));
                (Kind::Land, label, lb.hands, 0.0, Some(lb.good))
            }
            Key::OtherLand => {
                let ls: Vec<&accounts::LandBooks> = tb.land.iter().filter(|l| gathered.contains(&Key::Land(l.sector))).collect();
                let hands: f64 = ls.iter().map(|l| l.hands).sum();
                said.push(format!("The smaller uses of the land, together: {}; {} hands.", and_list(&ls.iter().map(|l| lower(&C.sector_label[l.sector])).collect::<Vec<_>>()), whole(hands)));
                (Kind::OtherLand, "Other land".to_string(), hands, 0.0, None)
            }
            Key::Works(g) => {
                let ms: Vec<&accounts::MethodBooks> = by_good[g].iter().map(|&i| &tb.methods[i]).collect();
                let (hands, works) = (ms.iter().map(|m| m.hands).sum::<f64>(), ms.iter().map(|m| m.works).sum::<f64>());
                let (out, value) = (ms.iter().map(|m| m.out).sum::<f64>(), ms.iter().map(|m| m.value).sum::<f64>());
                let label = if ms.len() == 1 { C.method_label[ms[0].method].clone() } else { format!("{} ({} kinds)", C.good_label[g], ms.len()) };
                said.push(format!("{}: {} standing (a mean over {year}), {} hands.", if ms.len() == 1 { label.clone() } else { format!("Works that make {}", gl(g)) }, works_n(works), whole(hands)));
                said.push(format!("Made {} {} in {year}, sold for {} ({} a unit); paid {} in wages and {} for materials and fuel.", qty(out), gl(g), whole(value), amount(if out > 0.0 { value / out } else { 0.0 }), whole(ms.iter().map(|m| m.wages).sum::<f64>()), whole(ms.iter().map(|m| m.inputs + m.fuel).sum::<f64>())));
                works_words(&[g], &mut said);
                let idle: Vec<String> = unused(g).iter().map(|&m| format!("{} ({})", C.method_label[m], recipe(C, m))).collect();
                if !idle.is_empty() {
                    said.push(format!("Known here and used by none: {}.", idle.join("; ")));
                }
                (Kind::Works, label, hands, works, Some(g))
            }
            Key::OtherWorks => {
                let gs: Vec<usize> = (0..G).filter(|&g| gathered.contains(&Key::Works(g))).collect();
                let hands: f64 = gs.iter().flat_map(|&g| by_good[g].iter()).map(|&i| tb.methods[i].hands).sum();
                let works: f64 = gs.iter().flat_map(|&g| by_good[g].iter()).map(|&i| tb.methods[i].works).sum();
                said.push(format!("The smaller kinds of works, together: {} standing, {} hands.", works_n(works), whole(hands)));
                works_words(&gs, &mut said);
                (Kind::OtherWorks, "Other works".to_string(), hands, works, None)
            }
            Key::Households => {
                let people = fig("pop");
                let spent: Vec<(f64, String)> = tb.needs.iter().map(|nb| (nb.households + nb.landlords, nb.name.clone())).filter(|x| x.0 > 0.0).collect();
                let all: f64 = spent.iter().map(|x| x.0).sum();
                let mut by: Vec<(f64, usize)> = tb.needs.iter().map(|nb| (nb.households + nb.landlords, nb.need)).filter(|x| x.0 > 0.0).collect();
                by.sort_by(|a, b| b.0.total_cmp(&a.0));
                let shares: Vec<String> = by.iter().take(5).map(|&(v, s)| format!("{} {}", lower(&C.service_label[s]), fmt_pct(v / all.max(1e-12), 0))).collect();
                said.push(format!("{} people at the year's end; their goods worth {} at the prices here in {year}.", whole(people), whole(value_in[i])));
                if !shares.is_empty() {
                    said.push(format!("Their spending by need: {}.", and_list(&shares)));
                }
                (Kind::Households, "Households".to_string(), people, 0.0, None)
            }
            Key::LandBuys => {
                said.push(format!("What the farms, woods and pits bought in {year}, worth {}: machines and what they run on, fertiliser, and rails for the pits.", whole(value_in[i])));
                (Kind::LandBuys, "Bought by farms and pits".to_string(), 0.0, 0.0, None)
            }
            Key::Building => {
                said.push(format!("Plant for works being built, and the materials of railways, canals and the university, in {year}: worth {}.", whole(value_in[i])));
                (Kind::Building, "Building".to_string(), 0.0, 0.0, None)
            }
        };
        nodes.push(Node { kind, label, col: col[i], hands, works, value_in: value_in[i], value_out: value_out[i], good, said });
    }

    // ---- what each ribbon says
    let recv_of = |g: usize| tb.received.iter().filter(|f| f.good == g).fold((0.0, 0.0, 0.0), |a, f| (a.0 + f.qty, a.1 + f.value, a.2 + f.carting));
    let sent_of = |g: usize| tb.sent.iter().filter(|f| f.good == g).fold((0.0, 0.0), |a, f| (a.0 + f.qty, a.1 + f.carting));
    for r in ribbons.iter_mut() {
        let (a, b) = (&nodes[r.from], &nodes[r.to]);
        let p = price(r.good);
        r.said.push(format!("{}, from {} to {}", C.good_label[r.good], lower_box(&a.label), lower_box(&b.label)));
        r.said.push(format!("{} a year in {year}, worth {} at {} a unit here.", qty(r.qty), whole(r.value), amount(p)));
        if keys[r.from] == Key::FromTowns {
            let (q, v, c) = recv_of(r.good);
            let share = if q > 0.0 { r.qty / q } else { 0.0 };
            let from: Vec<(f64, String)> = tb.received.iter().filter(|f| f.good == r.good).map(|f| (f.value, town_name(f.town))).collect();
            r.said.push(format!("{} of the {} that came in, from {}; worth {} where it left, and {} paid to carry it.", fmt_pct(share, 0), gl(r.good), largest(from, 3), whole(share * v), whole(share * c)));
        }
        if keys[r.to] == Key::ToTowns {
            let (q, c) = sent_of(r.good);
            let share = if q > 0.0 { r.qty / q } else { 0.0 };
            let to: Vec<(f64, String)> = tb.sent.iter().filter(|f| f.good == r.good).map(|f| (f.value, town_name(f.town))).collect();
            r.said.push(format!("Sold to {}; their merchants paid {} to carry it.", largest(to, 3), whole(share * c)));
        }
        // carting within the market area: a kind's carting of its materials,
        // shared by value among what it took in; the land's of its crop
        if let Key::Works(g) = keys[r.to] {
            let cart: f64 = by_good[g].iter().map(|&i| tb.methods[i].cart_in).sum();
            if cart > 0.0 && nodes[r.to].value_in > 0.0 {
                r.said.push(format!("About {} to carry it to the works.", whole(cart * r.value / nodes[r.to].value_in)));
            }
        }
        if let Key::Land(s) = keys[r.from] {
            let cart = tb.land[s].carting;
            if cart > 0.0 && nodes[r.from].value_out > 0.0 {
                r.said.push(format!("About {} to carry it to market.", whole(cart * r.value / nodes[r.from].value_out)));
            }
        }
        if r.back {
            r.said.push("It runs back against the chart's flow: a cycle in the town's making, cut here.".to_string());
        }
    }

    // ---- kinds the town could have and has not
    let (_, depth) = crate::atlas::depths(C);
    let mut faint = Vec::new();
    for g in 0..G {
        if !by_good[g].is_empty() {
            continue;
        }
        let ms: Vec<usize> = (0..C.M).filter(|&m| C.method_out[m] == g && knows(m)).collect();
        if ms.is_empty() {
            continue;
        }
        let d = ms.iter().map(|&m| depth[m]).min().unwrap_or(1).clamp(1, last - 1);
        let label = if ms.len() == 1 { C.method_label[ms[0]].clone() } else { format!("{} ({} kinds)", C.good_label[g], ms.len()) };
        let mut said = vec![format!("None stands here, though the town knows how: it would make {}.", gl(g))];
        said.extend(ms.iter().map(|&m| format!("{}: {}.", C.method_label[m], recipe(C, m))));
        faint.push(Faint { good: g, label, col: d, said });
    }

    let hands = fig("hands_land") + fig("hands_works");
    let said = format!("How {} made its living in {year}: a box for each kind of works and use of the land, as tall as the value through it and as wide as its hands; a ribbon for each good, as wide as its value at the town's prices. Each good is pooled in the town's market, so a ribbon is its maker's share of what the town had, times what its buyer took.", tb.name);
    Making {
        town: tb.town,
        name: tb.name.clone(),
        year,
        cols: last + 1,
        nodes,
        ribbons,
        faint,
        goods,
        good_label: C.good_label.clone(),
        good_colour: C.good_color.clone(),
        hands,
        said,
    }
}

fn works_n(n: f64) -> String {
    if (n - 1.0).abs() < 0.05 { "1 works".to_string() } else if n < 10.0 { format!("{n:.1} works") } else { format!("{} works", whole(n)) }
}

fn n_towns(n: usize) -> String {
    if n == 1 { "1 town".to_string() } else { format!("{n} towns") }
}

/// The towns of a list of flows, each with what it carried, worth more than nothing.
fn partners(fl: &[accounts::TownFlow], name: &dyn Fn(usize) -> String) -> Vec<(f64, String)> {
    let mut by: HashMap<usize, f64> = HashMap::new();
    for f in fl {
        *by.entry(f.town).or_default() += f.value;
    }
    by.into_iter().filter(|x| x.1 > 0.0).map(|(t, v)| (v, name(t))).collect()
}

/// A box's name inside a sentence: "the households", "other towns".
fn lower_box(label: &str) -> String {
    match label {
        "From other towns" | "To other towns" => "other towns".to_string(),
        "Households" => "the households".to_string(),
        "Out of store" | "Into store, or spoiled" => "the merchants' stocks".to_string(),
        "Bought by farms and pits" => "the farms and pits".to_string(),
        "Building" => "building".to_string(),
        _ => lower(label),
    }
}
