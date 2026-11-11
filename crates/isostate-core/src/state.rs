//! The simulation state: Python's `st` dict as one struct of arrays.
//!
//! Field names are the Python keys (`use` becomes `land_use`, a Rust keyword).
//! Per-town arrays have length `T_max`, per-cell arrays length `N`, per-firm arrays
//! length `F_max`; 2D arrays are `Mat` (row-major). The struct is built by
//! `sim::init_state`; every module reads and writes it through free functions.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::families::Family;
use crate::mat::Mat;
use crate::observer::{ObserverState, Series};
use crate::transport::Graph;
use crate::world::World;

/// The numbers that justified a firm, kept for the map inspector (`firms.reason_record`).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Reason {
    pub kind: String,
    pub year: i64,
    pub town: usize,
    pub cell: usize,
    pub K: f64,
    pub roi: f64,
    pub margin: f64,
    pub pout: f64,
    /// [good, units per unit of output, delivered price]
    pub inputs: Vec<(usize, f64, f64)>,
    /// [heat per unit, price per unit of heat]
    pub heat: [f64; 2],
    /// [worker-months per unit, wage]
    pub labor: [f64; 2],
    pub rent: f64,
    pub displaced: String,
    pub agglo: f64,
    pub dist: f64,
    pub n_cand: usize,
}

/// A railway under construction (a trunk line between two towns, or a mineral spur).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Project {
    pub a: usize,
    /// The other town; equal to `a` for a spur.
    pub b: usize,
    pub spur: bool,
    /// For a spur: the busy cell it serves.
    pub target: usize,
    /// The town whose market supplies the materials.
    pub town: usize,
    pub cells: Vec<usize>,
    pub new: Vec<usize>,
    pub need: Vec<f64>,
    pub order: Vec<f64>,
    pub cash: f64,
    pub start: i64,
    pub roi: f64,
    pub cost: f64,
    pub length_km: f64,
}

/// One opened line: (year, a, b, km) in Python; b is -1 for a spur.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct RailOpened {
    pub year: f64,
    pub a: usize,
    pub b: i64,
    pub km: f64,
}

/// A news item. Optional fields are left out of the JSON when absent; the field
/// order is the key order the Python dicts are built in.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Event {
    pub year: f64,
    pub kind: String,
    pub clock: String,
    pub weight: f64,
    pub title: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<[f64; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub town: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell2: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tech: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firm: Option<usize>,
    /// The family the news is about (`families`; never set without families).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<usize>,
}

impl Event {
    /// An event with the fields every event has; set the optional ones afterwards.
    pub fn new(year: f64, kind: &str, weight: f64, title: String, text: String) -> Event {
        Event { year, kind: kind.to_string(), clock: "live".to_string(), weight, title, text, ..Event::default() }
    }
}

/// What one month did, for the observer (`st["month_stats"]`).
#[derive(Clone, Debug, Default)]
pub struct MonthStats {
    pub prod: Vec<f64>,
    pub method_out: Vec<f64>,
    pub hh_got: Mat,
    pub lord_got: Mat,
    /// (T, G * S): share of each service bought from each good, `share[t][g * S + s]`.
    pub share: Mat,
    pub hh_spend: f64,
    pub lord_spend: f64,
    pub urban_rent: f64,
    pub employment: Vec<f64>,
    pub ex_labor_by_sector: Vec<f64>,
    pub firm_labor_by_method: Vec<f64>,
    pub heat_use: Vec<f64>,
    pub supply: Vec<f64>,
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pub world: World,
    pub N: usize,
    pub T_max: usize,

    // ---------------- towns
    pub town_active: Vec<bool>,
    pub town_node: Vec<usize>,
    pub town_founded: Vec<i64>,
    pub town_new: Vec<bool>,
    pub town_name: Vec<String>,
    pub used_names: BTreeSet<String>,
    pub pop: Vec<f64>,

    // ---------------- cells
    /// Dominant use for the map (`land::USE_*`); Python's `st["use"]`.
    pub land_use: Vec<u8>,
    pub cover: Vec<f64>,
    /// 0 none, 1 track, 2 road, 3 turnpike.
    pub road: Vec<u8>,
    pub rail: Vec<u8>,
    pub road_quiet: Vec<i64>,
    pub firm_at: Vec<i64>,
    pub cell_K_share: Vec<f64>,
    pub land_rent_only: Vec<f64>,
    pub derelict_timer: Vec<i64>,
    pub cell_rent: Vec<f64>,
    pub cell_flow: Vec<f64>,
    pub traffic: Vec<f64>,
    pub urban: Vec<bool>,
    pub urban_res: Vec<f64>,
    pub water: Vec<bool>,

    // ---------------- firms
    pub F_count: usize,
    pub f_alive: Vec<bool>,
    pub f_method: Vec<usize>,
    pub f_cell: Vec<usize>,
    pub f_town: Vec<usize>,
    pub f_born: Vec<i64>,
    pub f_died: Vec<i64>,
    pub f_loss_years: Vec<i64>,
    pub f_parent: Vec<i64>,
    pub f_K: Vec<f64>,
    pub f_order_K: Vec<f64>,
    pub f_cash: Vec<f64>,
    pub f_rent: Vec<f64>,
    pub f_u: Vec<f64>,
    pub f_fill: Vec<f64>,
    pub f_prod: Vec<f64>,
    pub f_profit_acc: Vec<f64>,
    pub f_out_acc: Vec<f64>,
    pub f_cost_m: Vec<f64>,
    pub f_labor: Vec<f64>,
    /// (F_max, G): capital goods still on order.
    pub f_order: Mat,
    pub f_name: Vec<String>,
    pub f_reason: Vec<Reason>,
    pub new_firms_year: Vec<usize>,
    pub closed_year: Vec<usize>,

    // ---------------- money and markets
    pub hh_cash: Vec<f64>,
    pub lord_cash: Vec<f64>,
    pub merch_cash: Vec<f64>,
    pub hh_income: Vec<f64>,
    pub lord_income: Vec<f64>,
    pub hh_income_prev: Vec<f64>,
    pub lord_income_prev: Vec<f64>,
    pub fund_cash: f64,
    pub issued: f64,
    /// (T, G)
    pub price: Mat,
    pub pexp: Mat,
    /// (NS + 1, N): each cell's land shares by sector; the last row is wild land.
    pub lam: Mat,
    pub inv: Mat,
    pub outflow: Mat,
    pub fill: Mat,
    pub wage: Vec<f64>,
    pub tools_mult: Vec<f64>,
    pub imp_pc: Vec<f64>,
    pub urban_rent: Vec<f64>,
    pub urban_pop: Vec<f64>,
    pub ag_labor: Vec<f64>,
    pub ind_labor: Vec<f64>,
    pub y_ratio: Vec<f64>,

    // ---------------- knowledge
    /// (T, NT)
    pub know: Mat,
    pub tech_year: Vec<i64>,
    pub tech_town: Vec<i64>,
    pub t_pump: usize,
    pub t_wagon: usize,
    pub t_rail: usize,
    pub t_husb: usize,
    pub t_fmach: usize,
    pub forestry_sector: usize,
    pub method_Q: Vec<f64>,
    pub method_A: Vec<f64>,

    // ---------------- accumulators
    pub y_acc: Vec<f64>,
    pub price_acc: Mat,
    pub wage_acc: Vec<f64>,
    pub town_prod: Mat,
    pub town_prod_year: Mat,
    pub cumprod: Vec<f64>,
    /// (T, T): weight carted from town i to town j this year.
    pub trade_F: Mat,
    pub trade_F_year: Mat,
    pub food_min_acc: Vec<f64>,
    pub projects: Vec<Project>,
    pub rail_links: BTreeSet<(usize, usize)>,
    pub rail_cooldown: BTreeMap<(usize, usize), i64>,
    pub spur_done: BTreeSet<usize>,
    pub cell_flow_year: Vec<f64>,
    pub rail_opened: Vec<RailOpened>,
    pub m_rent: f64,
    pub m_invest: f64,
    pub m_merch_margin: f64,
    pub m_trade_value: f64,

    // ---------------- extractive units (cell, sector) used by the monthly loop
    pub ex_cell: Vec<usize>,
    pub ex_sector: Vec<usize>,
    pub ex_town: Vec<usize>,
    pub ex_lam: Vec<f64>,
    pub ex_field: Vec<f64>,
    pub ex_tau: Vec<f64>,
    pub ex_Lprev: Vec<f64>,
    pub ex_labor: Vec<f64>,
    pub ex_out: Vec<f64>,

    // ---------------- transport (land::update_transport)
    pub graph: Graph,
    /// (T, N): path cost from each town to every cell.
    pub dist: Mat,
    /// Predecessor tree of each town's shortest paths (`transport::NO_PRED` at roots;
    /// empty for inactive towns).
    pub pred: Vec<Vec<i64>>,
    /// The cells each town's search settled, nearest first (not in Python, which
    /// sorts by distance where it needs this order).
    pub settle: Vec<Vec<u32>>,
    /// Market town of each cell, -1 for none.
    pub catch: Vec<i64>,
    pub dcatch: Vec<f64>,
    pub town_cost: Mat,
    /// (G, N): delivered price of each good at each cell relative to its town price.
    pub delta: Mat,

    // ---------------- sim::after_transport and firms::update_productivity
    pub town_cells: Vec<Vec<usize>>,
    pub any_counts: Vec<f64>,
    pub cell_firms: Vec<f64>,
    /// (M, N)
    pub same_counts: Mat,

    // ---------------- land::update_land_use
    /// (NS, N)
    pub bids: Mat,
    pub cell_urban_rent: Vec<f64>,
    pub urban_density: f64,

    // ---------------- written by econ::step_month
    pub month_stats: MonthStats,
    pub subcost: Vec<f64>,
    pub food_ratio: Vec<f64>,

    // ---------------- written by sim::yearly
    pub pbar: Mat,
    pub wbar: Vec<f64>,
    pub prev_catch: Vec<i64>,
    pub cell_labor: Vec<f64>,
    pub migration_flow: Mat,

    // ---------------- written by sim::run
    pub cumprod_1750: Vec<f64>,
    /// (NT, G): discovery thresholds with the "xN" multiples resolved.
    pub disc_X0: Mat,
    pub cpi_basket: Vec<f64>,
    pub cpi_base_prices: Vec<f64>,
    pub wage_base: f64,
    pub money_start: f64,
    pub money_now: f64,

    // ---------------- the observer's registry and memory (never read by the sim)
    pub series: Series,
    pub obs: ObserverState,

    // ---------------- families (not in the Python; all empty when `n_families` is 0)
    pub families: Vec<Family>,
    /// Owning family of each firm slot, or -1 for the anonymous small capital.
    pub f_owner: Vec<i64>,
    /// Money each family works has paid its family so far: dividends, and its
    /// cash when it closed.
    pub f_paid: Vec<f64>,
    /// Owning family of each cell, or -1 for the anonymous gentry.
    pub cell_owner: Vec<i64>,
    /// Rent each cell has paid this year so far, whoever received it.
    pub cell_rent_paid: Vec<f64>,
    /// Rent each cell paid over the last complete year (what land is priced on).
    pub cell_rent_year: Vec<f64>,
    /// (T, G): what each town's buyers asked for last month (households, landlords,
    /// works, mines, building and railway orders), whether or not they got it.
    pub requests: Mat,

    // ---------------- machines (not in the Python; empty while `machines.enabled` is off)
    /// (T, NS * machines::SLOTS): the share of each land sector's output in each
    /// town worked by machines now standing, one column per use (`machines::uses`).
    pub machine_stock: Mat,
}
