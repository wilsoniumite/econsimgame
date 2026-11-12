//! A live view of a running simulation, for the viewer: what never changes, once
//! (`live_static`), and after every simulated year what that year changed
//! (`live_year`). Not in the Python; `record` keeps the same data for a whole
//! run, and the encodings are the same as in `run.bin`. A viewer whose clock
//! shows months may also ask after each month for the little that the months
//! change (`live_month`: the series, the towns' prices and wages, the
//! families' cash, a railway opened).
//!
//! The intended loop, in the thread that owns the `Sim`:
//!
//! ```no_run
//! use isostate_core::{default_content, default_params, live, sim};
//! let mut P = default_params();
//! P.n_families = 6;
//! let mut s = sim::start(&P, default_content());
//! let stat = live::live_static(&s.st, &s.content);
//! let mut cursor = live::LiveCursor::new(&s.content);
//! while s.year <= s.end_year {
//!     sim::run_year(&mut s).unwrap();
//!     let year = live::live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events);
//!     // send `year` to the viewer
//! #   let _ = (&stat, year);
//! }
//! ```

use serde::Serialize;

use crate::content::Content;
use crate::families::{self, FamilyReport, Group};
use crate::land::USE_NAMES;
use crate::observer::annual;
use crate::params::Params;
use crate::record::{self, DISP, LAYERS, ProjectSnap, STATIC_FIELDS};
use crate::state::{Event, Reason, State};

#[derive(Clone, Debug, Serialize)]
pub struct LiveGood {
    pub name: String,
    pub label: String,
    pub color: String,
    pub weight: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct LiveMethod {
    pub name: String,
    pub label: String,
    /// The good it makes.
    pub out: usize,
    /// Capacity of a full-size works.
    pub K0: f64,
    /// The industry group a family member can manage it in (None for shops and trades).
    pub group: Option<Group>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LiveTech {
    pub name: String,
    pub label: String,
    /// "method" or "boost".
    pub kind: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct LiveSeries {
    pub name: String,
    pub label: String,
    pub group: String,
    pub unit: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct LiveGroup {
    pub name: String,
    pub label: String,
}

/// Everything that never changes during a run. Per-cell arrays are row-major,
/// `H` rows of `W` cells.
#[derive(Clone, Debug, Serialize)]
pub struct LiveStatic {
    pub W: usize,
    pub H: usize,
    pub start_year: i64,
    pub end_year: i64,
    pub km_per_cell: f64,
    /// Height, 0 to 255.
    pub elev: Vec<u8>,
    /// `world::SEA`, `LOWLAND`, `HILLS` or `MOUNTAINS`.
    pub terrain: Vec<u8>,
    /// 0 for no river, else 1 plus 20 times log2 of the upstream area.
    pub river: Vec<u8>,
    /// 1 where a river is navigable.
    pub nav: Vec<u8>,
    /// The cell water flows to, or -1.
    pub down: Vec<i32>,
    /// The resource fields, 0 to 255: `STATIC_FIELDS` order, names in `field_names`.
    pub fields: Vec<Vec<u8>>,
    pub field_names: Vec<String>,
    pub goods: Vec<LiveGood>,
    pub methods: Vec<LiveMethod>,
    pub techs: Vec<LiveTech>,
    /// Land sector names (the land units of the economy).
    pub sectors: Vec<String>,
    /// Names of the `land_use` codes.
    pub uses: Vec<String>,
    /// Names of the display classes in `mix_a` and `mix_b`.
    pub disp: Vec<String>,
    /// Names of the nine yearly layers, in `LiveLayers` field order.
    pub layers: Vec<String>,
    /// The monthly series whose annual means `LiveYear::series` holds, in that order.
    pub series: Vec<LiveSeries>,
    /// The industry groups, in `Group::index` order.
    pub groups: Vec<LiveGroup>,
}

/// The static half. Call after `sim::start` (the series registry exists from then).
pub fn live_static(st: &State, C: &Content) -> LiveStatic {
    let w = &st.world;
    LiveStatic {
        W: w.W,
        H: w.H,
        start_year: C.start_year,
        end_year: C.end_year,
        km_per_cell: 2.0,
        elev: record::elev_bytes(w),
        terrain: w.terrain.clone(),
        river: record::river_bytes(w),
        nav: w.navigable.iter().map(|&n| n as u8).collect(),
        down: w.down.iter().map(|&d| d as i32).collect(),
        fields: STATIC_FIELDS.iter().map(|f| record::field_bytes(w.field(f))).collect(),
        field_names: STATIC_FIELDS.iter().map(|f| f.to_string()).collect(),
        goods: (0..C.G)
            .map(|g| LiveGood {
                name: C.good_name[g].clone(),
                label: C.good_label[g].clone(),
                color: C.good_color[g].clone(),
                weight: C.weight[g],
            })
            .collect(),
        methods: (0..C.M)
            .map(|m| LiveMethod {
                name: C.method_name[m].clone(),
                label: C.method_label[m].clone(),
                out: C.method_out[m],
                K0: C.m_K0[m],
                group: Group::of_method(C, m),
            })
            .collect(),
        techs: (0..C.NT)
            .map(|t| LiveTech { name: C.tech_name[t].clone(), label: C.tech_label[t].clone(), kind: C.tech_kind[t].clone() })
            .collect(),
        sectors: C.sector_name.clone(),
        uses: USE_NAMES.iter().map(|s| s.to_string()).collect(),
        disp: DISP.iter().map(|s| s.to_string()).collect(),
        // towns: the three it adds are named only where the game keeps town ground
        layers: LAYERS.iter().chain(record::TOWN_LAYERS.iter().filter(|_| crate::towns::kept(st))).map(|s| s.to_string()).collect(),
        series: st
            .series
            .names
            .iter()
            .zip(&st.series.meta)
            .map(|(n, m)| LiveSeries { name: n.clone(), label: m.label.clone(), group: m.group.clone(), unit: m.unit.clone() })
            .collect(),
        groups: Group::ALL.iter().map(|g| LiveGroup { name: g.name().to_string(), label: g.label().to_string() }).collect(),
    }
}

/// The nine map layers of one year, one byte per cell (as in `run.bin`).
#[derive(Clone, Debug, Serialize)]
pub struct LiveLayers {
    /// `land::USE_*` code (names in `LiveStatic::uses`).
    pub land_use: Vec<u8>,
    /// How intensively the cell is used, 0 to 255.
    pub inten: Vec<u8>,
    /// Rent, log scale: 0 for none, else 255 * (log10(rent) + 3) / 6.
    pub rent: Vec<u8>,
    /// Road level 0 to 3, plus 4 for a railway, plus 8 for a railway being built.
    pub trans: Vec<u8>,
    /// Catchment: town index plus 1 (0 for none).
    pub catch: Vec<u8>,
    /// Distance to the town, in quarter units of path cost (255 for unreachable).
    pub dist: Vec<u8>,
    /// The two largest display classes and the share of the first (127 to 255).
    pub mix_a: Vec<u8>,
    pub mix_b: Vec<u8>,
    pub mix_s: Vec<u8>,
    // towns: three more, only in a game whose towns keep their ground
    // (`record::TOWN_LAYERS`, the same bytes as in `run.bin`)
    /// The year the cell joined its town less `record::BUILT_YEAR0` (1 to 255); 0: not town ground.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub built: Option<Vec<u8>>,
    /// The cell's residents, in units of `record::RES_UNIT` people.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub res: Option<Vec<u8>>,
    /// The share of the cell under works, 0 to 255, on every cell.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub works: Option<Vec<u8>>,
}

/// Town arrays for the year, indexed by town slot (`T_max` of them).
#[derive(Clone, Debug, Serialize)]
pub struct LiveTowns {
    pub active: Vec<bool>,
    pub pop: Vec<f32>,
    pub urban: Vec<f32>,
    /// The year's mean wage.
    pub wage: Vec<f32>,
    /// The year's mean prices, `[town * G + good]`.
    pub price: Vec<f32>,
    /// Knowledge of each tech, 0 to 255, `[town * NT + tech]`.
    pub know: Vec<u8>,
}

/// A firm the viewer has not been told about yet.
#[derive(Clone, Debug, Serialize)]
pub struct LiveFirm {
    pub id: usize,
    pub name: String,
    pub method: usize,
    pub cell: usize,
    pub town: usize,
    pub born: i64,
    /// The firm it was a sister works of.
    pub parent: Option<usize>,
    /// The owning family (None: anonymous small capital).
    pub owner: Option<usize>,
    /// The numbers that justified it (`Reason::kind` is "old", "new", "expansion" or "family").
    pub reason: Reason,
}

#[derive(Clone, Debug, Serialize)]
pub struct LiveTown {
    pub id: usize,
    pub name: String,
    /// The town's cell.
    pub node: usize,
    pub founded: i64,
}

/// One finished year.
#[derive(Clone, Debug, Serialize)]
pub struct LiveYear {
    pub year: i64,
    /// `year - start_year`, the year's index in `run.bin`.
    pub index: usize,
    pub layers: LiveLayers,
    pub towns: LiveTowns,
    /// (firm id, capacity) of every firm alive at the end of the year.
    pub firm_K: Vec<(usize, f32)>,
    /// Firms created since the previous call (all firms on the first call).
    pub new_firms: Vec<LiveFirm>,
    /// Firms closed since the previous call.
    pub closed_firms: Vec<usize>,
    /// Towns founded since the previous call (all towns on the first call).
    pub new_towns: Vec<LiveTown>,
    /// Railways under construction: towns and the share done.
    pub projects: Vec<ProjectSnap>,
    /// News since the previous call.
    pub events: Vec<Event>,
    /// Chronicle: news sent in earlier calls whose `kept` mark has changed since,
    /// as (place in the run's event list, the mark now). The places count the
    /// events in the order they were sent. Empty without the chronicle.
    pub rekept: Vec<(usize, bool)>,
    /// The year's mean of every monthly series, in `LiveStatic::series` order.
    pub series: Vec<f64>,
    /// Every family's report (empty without families).
    pub families: Vec<FamilyReport>,
    /// Owning family of every cell, -1 for none (empty without families).
    pub cell_owner: Vec<i16>,
    /// harvest: each town's harvest this year against an average year, by town
    /// slot (1.0 is average; empty unless the mechanism is on), and the country's.
    pub harvest_town: Vec<f32>,
    pub harvest: Option<f32>,
    /// mines: every working there is or has been, as the year ends (empty
    /// unless the mechanism is on; docs/rules/mines.md).
    pub workings: Vec<crate::mines::Working>,
    /// canals (docs/rules/canals.md): every link of canal so far, from cell to
    /// cell, with the year its canal was begun and the moment it opened (absent
    /// while it is being dug). Empty unless the mechanism is on. The `trans`
    /// layer marks the same cells with 16 (open) and 32 (digging).
    pub canals: Vec<crate::canals::LinkOut>,
    /// accounts: the island's income account for the year (None unless the books
    /// are kept). One town's or one works' books are asked for when wanted, with
    /// `accounts::town_books` and `accounts::works_books`.
    pub accounts: Option<crate::accounts::IslandYear>,
}

/// One finished month: what the economy changes month by month, kept small
/// (a few hundred numbers), for a viewer whose clock shows months. The map,
/// the works, the families' reports and most of the news are the year's
/// (`LiveYear`).
#[derive(Clone, Debug, Serialize)]
pub struct LiveMonth {
    pub year: i64,
    /// 0 for January.
    pub month: usize,
    /// The month's place in the monthly series: 12 times the year's index, plus the month.
    pub index: usize,
    /// The month's value of every monthly series, in `LiveStatic::series` order.
    pub series: Vec<f64>,
    /// The month's wage in each town, by town slot.
    pub wage: Vec<f32>,
    /// The month's prices, `[town * G + good]`.
    pub price: Vec<f32>,
    /// Every family's cash at the month's end (empty without families).
    pub cash: Vec<f64>,
    /// News since the previous call (`live_month` or `live_year`): what the
    /// months themselves make, which is a railway opened.
    pub events: Vec<Event>,
}

/// The values of every monthly series in month `t` of the run (0 for the first
/// January), in `LiveStatic::series` order.
pub fn month_series(st: &State, t: usize) -> Vec<f64> {
    st.series.data.iter().map(|d| d.get(t).copied().unwrap_or(f64::NAN)).collect()
}

/// The month just simulated. Call after `sim::run_month` (or `save::Game::month`)
/// with `t` the month's place in the series, which is `Sim::t - 1` then; the
/// year's last month may be left to `live_year`, which sends the news either way.
pub fn live_month(cursor: &mut LiveCursor, st: &State, C: &Content, P: &Params, events: &[Event], t: usize) -> LiveMonth {
    let (T, G) = (st.T_max, C.G);
    let new_events = events[cursor.events.min(events.len())..].to_vec();
    // chronicle: the marks these went out with, so a later change to one is told
    if P.chronicle.enabled {
        cursor.kept.extend(new_events.iter().map(|e| e.kept));
    }
    cursor.events = events.len();
    LiveMonth {
        year: C.start_year + (t / 12) as i64,
        month: t % 12,
        index: t,
        series: month_series(st, t),
        wage: (0..T).map(|i| if st.town_active[i] { st.wage[i] as f32 } else { 0.0 }).collect(),
        price: (0..T * G).map(|k| st.price.data[k] as f32).collect(),
        cash: st.families.iter().map(|f| f.cash).collect(),
        events: new_events,
    }
}

/// What the viewer has been sent so far.
#[derive(Clone, Debug)]
pub struct LiveCursor {
    /// The year the next `live_year` reports.
    pub next_year: i64,
    firms: usize,
    towns: usize,
    events: usize,
    alive: Vec<bool>,
    /// Chronicle: the `kept` mark each event had when it was last sent.
    kept: Vec<Option<bool>>,
}

impl LiveCursor {
    /// A cursor that has sent nothing; its first `live_year` reports the first year.
    pub fn new(C: &Content) -> LiveCursor {
        LiveCursor { next_year: C.start_year, firms: 0, towns: 0, events: 0, alive: Vec::new(), kept: Vec::new() }
    }
}

/// The year just simulated, and what changed since the previous call. Call once
/// after each `sim::run_year`; `events` is the run's event list (`Sim::events`).
pub fn live_year(cursor: &mut LiveCursor, st: &State, C: &Content, P: &Params, events: &[Event]) -> LiveYear {
    let year = cursor.next_year;
    cursor.next_year += 1;
    let (T, G, NT) = (st.T_max, C.G, C.NT);

    let mut l = record::layers(st, C).into_iter();
    let mut next = || l.next().expect("nine layers");
    let mut tl = record::town_layers(st).unwrap_or_default().into_iter();
    let layers = LiveLayers {
        land_use: next(),
        inten: next(),
        rent: next(),
        trans: next(),
        catch: next(),
        dist: next(),
        mix_a: next(),
        mix_b: next(),
        mix_s: next(),
        built: tl.next(),
        res: tl.next(),
        works: tl.next(),
    };
    let towns = LiveTowns {
        active: st.town_active.clone(),
        pop: st.pop.iter().map(|&p| p as f32).collect(),
        urban: (0..T).map(|t| if st.town_active[t] { st.urban_pop[t] as f32 } else { 0.0 }).collect(),
        wage: st.wbar.iter().map(|&w| w as f32).collect(),
        price: (0..T * G).map(|k| st.pbar.data[k] as f32).collect(),
        know: (0..T * NT).map(|k| record::know_byte(st.know.data[k])).collect(),
    };

    let firm_K: Vec<(usize, f32)> = (0..st.F_count).filter(|&f| st.f_alive[f]).map(|f| (f, st.f_K[f] as f32)).collect();
    let new_firms: Vec<LiveFirm> = (cursor.firms..st.F_count)
        .map(|f| LiveFirm {
            id: f,
            name: st.f_name[f].clone(),
            method: st.f_method[f],
            cell: st.f_cell[f],
            town: st.f_town[f],
            born: st.f_born[f],
            parent: (st.f_parent[f] >= 0).then_some(st.f_parent[f] as usize),
            owner: families::owner_of_firm(st, f),
            reason: st.f_reason[f].clone(),
        })
        .collect();
    let closed_firms: Vec<usize> = (0..cursor.alive.len()).filter(|&f| cursor.alive[f] && !st.f_alive[f]).collect();
    cursor.firms = st.F_count;
    cursor.alive = st.f_alive[..st.F_count].to_vec();

    let new_towns: Vec<LiveTown> = (cursor.towns..st.town_name.len())
        .map(|i| LiveTown { id: i, name: st.town_name[i].clone(), node: st.town_node[i], founded: st.town_founded[i] })
        .collect();
    cursor.towns = st.town_name.len();

    let new_events = events[cursor.events.min(events.len())..].to_vec();
    // chronicle: a decade is ranked again as it fills, so marks already sent can change
    let rekept: Vec<(usize, bool)> = if P.chronicle.enabled {
        (0..cursor.kept.len().min(events.len())).filter(|&i| events[i].kept != cursor.kept[i]).map(|i| (i, events[i].kept == Some(true))).collect()
    } else {
        Vec::new()
    };
    if P.chronicle.enabled {
        cursor.kept = events.iter().map(|e| e.kept).collect();
    }
    cursor.events = events.len();

    let projects = st
        .projects
        .iter()
        // canals: a canal being dug is not on the list of railways building
        .filter(|pr| pr.way != crate::ways::CANAL)
        .map(|pr| {
            let order: f64 = crate::util::pairwise_sum(&pr.order);
            let need: f64 = crate::util::pairwise_sum(&pr.need);
            ProjectSnap { a: pr.a, b: pr.b, done: 1.0 - order / need.max(1e-9) }
        })
        .collect();

    LiveYear {
        year,
        index: (year - C.start_year).max(0) as usize,
        layers,
        towns,
        firm_K,
        new_firms,
        closed_firms,
        new_towns,
        projects,
        events: new_events,
        rekept,
        series: st.series.data.iter().map(|d| annual(d, C.start_year, year)).collect(),
        families: (0..st.families.len()).map(|k| families::family_report(st, C, P, k).expect("family exists")).collect(),
        cell_owner: st.cell_owner.iter().map(|&o| o as i16).collect(),
        harvest_town: st.harvest_town.iter().map(|&v| v as f32).collect(),
        harvest: (!st.harvest_town.is_empty()).then_some(st.harvest_year as f32),
        workings: crate::mines::workings(st),
        canals: if P.canals.enabled { crate::canals::links_out(st) } else { Vec::new() },
        accounts: crate::accounts::island_year(st).filter(|y| y.year == year),
    }
}
