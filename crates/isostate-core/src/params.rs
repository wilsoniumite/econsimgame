//! Default parameters. Everything tunable lives here as one plain struct. Python: `params.py`.
//!
//! The field order is the order of Python's `default_params` dict, so `params.json`
//! comes out the same. Overrides are partial TOML or JSON documents: any field left
//! out keeps its default, and an unknown field is an error (Python would silently
//! store a misspelt key).
//!
//! # Two sets of rules
//!
//! [`default_params`] is the port: a run under it is the Python's run, byte for
//! byte, and must stay so (`tests/golden.rs` holds it to that). [`game_params`] is
//! what a live game is played under: the port plus every mechanism the Python never
//! had. `docs/rules/README.md` is the longer account.
//!
//! # Adding a mechanism
//!
//! A mechanism that is not in the Python keeps its numbers in its own small struct
//! in its own module. The struct derives `Clone, Debug, PartialEq, Serialize,
//! Deserialize` and carries `#[serde(default, deny_unknown_fields)]`; its `Default`
//! is the mechanism switched off, and a `game()` constructor gives the tuned values
//! a game is played with. `Params` gets one field for it, marked
//! `#[serde(default, skip_serializing_if = "is_default")]`, so the `params.json` of
//! a port run does not change. `default_params` leaves the field at its default and
//! `game_params` sets it to `game()`. Switched off, the mechanism must do nothing
//! at all: no change to the state, no draw from a random stream the port uses.
//!
//! A worked example, for canals:
//!
//! ```text
//! // canals.rs
//! #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
//! #[serde(default, deny_unknown_fields)]
//! pub struct CanalParams {
//!     /// Promoters may dig canals.
//!     pub enabled: bool,
//!     /// Cost of carrying a ton a cell by canal, relative to a turnpike.
//!     pub cost: f64,
//! }
//!
//! impl Default for CanalParams {
//!     /// Off: what the port runs with.
//!     fn default() -> CanalParams {
//!         CanalParams { enabled: false, cost: 0.3 }
//!     }
//! }
//!
//! impl CanalParams {
//!     /// The game's canals.
//!     pub fn game() -> CanalParams {
//!         CanalParams { enabled: true, ..CanalParams::default() }
//!     }
//! }
//!
//! // params.rs, at the end of `Params`
//!     /// Canals (not in the Python; see docs/rules/canals.md).
//!     #[serde(default, skip_serializing_if = "is_default")]
//!     pub canals: CanalParams,
//!
//! // in default_params():   canals: CanalParams::default(),
//! // in game_params():      P.canals = CanalParams::game();
//! ```
//!
//! The sweep can then set `--set canals.enabled=true` or `--set canals.cost=0.2,0.3,0.4`
//! with no further wiring. The family fields below predate this convention and stay
//! as they are.

use serde::{Deserialize, Serialize};

use crate::works::WorksParams;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RentRegime {
    /// Land rent goes to landlords, who spend more of it on shelter, services and comfort.
    Landlords,
    /// The same rent is paid out to every household as an equal dividend.
    Commons,
}

impl RentRegime {
    pub fn as_str(self) -> &'static str {
        match self {
            RentRegime::Landlords => "landlords",
            RentRegime::Commons => "commons",
        }
    }

    pub fn parse(s: &str) -> Option<RentRegime> {
        match s {
            "landlords" => Some(RentRegime::Landlords),
            "commons" => Some(RentRegime::Commons),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Params {
    pub seed: u64,
    pub W: usize,
    pub H: usize,
    pub n_towns: usize,
    pub pop0: i64,
    pub T_max: usize,
    pub F_max: usize,
    pub warmup_years: i64,
    // people
    pub participation: f64,
    pub growth_slope: f64,
    pub growth_base: f64,
    pub growth_cap: f64,
    pub migration_rate: f64,
    pub migration_dist: f64,
    // transport
    pub freight: f64,
    pub handling: f64,
    pub trade_k: f64,
    pub trade_cap: f64,
    pub road_thresholds: [f64; 3],
    // markets
    pub logit_theta: f64,
    pub heat_theta: f64,
    pub price_speed: f64,
    pub price_flow: f64,
    pub labor_adjust_ex: f64,
    pub labor_adjust_firm: f64,
    pub price_target_months: f64,
    pub price_floor: f64,
    /// Monthly inventory decay, one rate per good.
    pub decay: Vec<f64>,
    pub price_ceiling: f64,
    // households
    pub cons_income: f64,
    pub cons_wealth: f64,
    pub lord_share: f64,
    pub lord_cons_income: f64,
    pub lord_cons_wealth: f64,
    pub urban_rent_rate: f64,
    pub tools_boost: f64,
    pub tools_scale: f64,
    // money and ownership
    pub money_growth: f64,
    pub rent_regime: RentRegime,
    pub firm_div_rate: f64,
    pub firm_buffer_months: f64,
    pub merchant_spread: f64,
    pub merch_div_rate: f64,
    pub merch_buffer_months: f64,
    // firms
    pub hurdle: f64,
    pub invest_frac_hh: f64,
    pub invest_frac_lord: f64,
    pub util_target_margin: f64,
    pub util_band: f64,
    pub depreciation: f64,
    pub entry_K_frac: f64,
    pub cluster_K_mult: f64,
    pub footprint_K_mult: f64,
    pub agglo_same: f64,
    pub candidates: usize,
    // mines
    pub pump_input: f64,
    pub pump_boost: f64,
    pub wagon_input: f64,
    pub wagon_boost: f64,
    pub husbandry_boost: f64,
    pub farm_mach_boost: f64,
    pub farm_mach_input: f64,
    // land market
    pub land_theta: f64,
    pub land_adjust: f64,
    pub land_fixed_cost: f64,
    pub expect_rate: f64,
    // forests
    pub forest_capacity: f64,
    pub forest_regrow: f64,
    pub clearing_timber: f64,
    // tech
    pub discovery_rate: f64,
    pub diffusion_rate: f64,
    pub diffusion_dist: f64,
    pub learn_cap: f64,
    // railways
    pub rail_rails_per_cell: f64,
    pub rail_timber_per_cell: f64,
    pub rail_stone_per_cell: f64,
    pub rail_hurdle: f64,
    pub rail_induced: f64,
    pub rail_max_projects: usize,
    pub rail_know: f64,
    pub spur_min_flow: f64,
    // towns
    pub found_activity: f64,
    pub found_distance: f64,
    pub urban_density0: f64,
    pub urban_density_growth: f64,
    // detectors
    pub crossing_lo: f64,
    pub crossing_hi: f64,
    // families (not in the Python). Each is written to params.json only when it
    // differs from its default, so a run without families writes Python's file.
    /// Number of families, the player's (index 0) included; 0 turns families off.
    #[serde(skip_serializing_if = "is_zero")]
    pub n_families: usize,
    /// Cut in inputs, heat and labor per unit of a family's works in a group managed by a member of skill 1.
    #[serde(skip_serializing_if = "is_default_bonus")]
    pub family_bonus: f64,
    /// Share of last month's dividends and rents a family spends.
    #[serde(skip_serializing_if = "is_default_cons_income")]
    pub family_cons_income: f64,
    /// Share of its cash a family spends each month (land and works are not drawn on).
    #[serde(skip_serializing_if = "is_default_cons_wealth")]
    pub family_cons_wealth: f64,
    /// Land sells at this many years of its rent.
    #[serde(skip_serializing_if = "is_default_years_purchase")]
    pub years_purchase: f64,
    /// Rent counted for the price of land is at least this many town wages a month.
    #[serde(skip_serializing_if = "is_default_rent_floor")]
    pub land_rent_floor: f64,
    /// Each AI family starts with this share of all rent (landed) or of all works' capital (industrial).
    #[serde(skip_serializing_if = "is_default_start_share")]
    pub family_start_share: f64,
    /// Each family starts with this share of the money stock in cash.
    #[serde(skip_serializing_if = "is_default_start_cash")]
    pub family_start_cash: f64,
    /// Growing branches keep this many months of the family's spending back as a cash reserve.
    #[serde(skip_serializing_if = "is_default_reserve")]
    pub family_reserve_months: f64,
    /// AI families build only where the works' output is at most this share of the town's demand...
    #[serde(skip_serializing_if = "is_default_max_share")]
    pub family_max_share: f64,
    /// ...and the good's price is at most this many times its base price.
    #[serde(skip_serializing_if = "is_default_max_price_ratio")]
    pub family_max_price_ratio: f64,
    /// The head's bonus at the seat, at skill 1 (a manager's is `family_bonus`).
    #[serde(skip_serializing_if = "is_default_seat_bonus")]
    pub family_seat_bonus: f64,
    /// Yearly chance that an adult of 22 to 42 has a child.
    #[serde(skip_serializing_if = "is_default_birth_rate")]
    pub family_birth_rate: f64,
    /// Share of a family works' profit a hired manager takes when no branch runs it.
    #[serde(skip_serializing_if = "is_default_absentee_cut")]
    pub absentee_cut: f64,
    /// Skill gained at university.
    #[serde(skip_serializing_if = "is_default_university_gain")]
    pub university_gain: f64,
    /// Last simulated year; `None` means the content's end year (Python: `P.get("end_year", C["end_year"])`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_year: Option<i64>,
    /// Merchants who mark prices with the stock they hold (not in the Python; see docs/rules/prices.md).
    #[serde(default, skip_serializing_if = "is_default")]
    pub prices: crate::prices::PriceParams,
    /// An Engel curve in what households buy (not in the Python; see docs/rules/engel.md).
    #[serde(default, skip_serializing_if = "is_default")]
    pub engel: crate::engel::EngelParams,
    /// Machines bought where they pay, that improve, and steel that matters (not in
    /// the Python; see docs/rules/machines.md).
    #[serde(default, skip_serializing_if = "is_default")]
    pub machines: crate::machines::MachineParams,
    /// Good years and bad (not in the Python; see docs/rules/harvest.md).
    #[serde(default, skip_serializing_if = "is_default")]
    pub harvest: crate::harvest::HarvestParams,
    /// The chronicle: news that reads like a history (not in the Python; see docs/rules/chronicle.md).
    #[serde(default, skip_serializing_if = "is_default")]
    pub chronicle: crate::chronicle::ChronicleParams,
    /// A life cycle for works (not in the Python; see docs/rules/works.md).
    #[serde(default, skip_serializing_if = "is_default")]
    pub works: WorksParams,
    /// Branch lines priced on what they save, at a believable cost (not in the
    /// Python; see docs/rules/railways.md).
    #[serde(default, skip_serializing_if = "is_default")]
    pub railways: crate::railways::RailwayParams,
    /// Carting takes labour (not in the Python; see docs/rules/carting.md).
    #[serde(default, skip_serializing_if = "is_default")]
    pub carting: crate::carting::CartingParams,
    /// People (not in the Python; see docs/rules/people.md): a population that
    /// answers to how people live, and towns founded all century.
    #[serde(default, skip_serializing_if = "is_default")]
    pub people: crate::peopling::PeopleParams,
    /// Land (not in the Python; see docs/rules/land.md): an island made by
    /// uplift and erosion, with its water, in place of the port's.
    #[serde(default, skip_serializing_if = "is_default")]
    pub land: crate::landform::LandParams,
}

const FAMILY_BONUS: f64 = 0.08;
const FAMILY_CONS_INCOME: f64 = 0.7;
const FAMILY_CONS_WEALTH: f64 = 0.01;
const YEARS_PURCHASE: f64 = 25.0;
const LAND_RENT_FLOOR: f64 = 0.5;
const FAMILY_START_SHARE: f64 = 0.03;
const FAMILY_START_CASH: f64 = 0.004;
const FAMILY_RESERVE_MONTHS: f64 = 3.0;
const FAMILY_MAX_SHARE: f64 = 0.25;
const FAMILY_MAX_PRICE_RATIO: f64 = 3.0;
const FAMILY_SEAT_BONUS: f64 = 0.12;
const FAMILY_BIRTH_RATE: f64 = 0.15;
const ABSENTEE_CUT: f64 = 0.2;
const UNIVERSITY_GAIN: f64 = 0.25;

/// For `skip_serializing_if` on a mechanism's field: true while it is switched
/// off, so the `params.json` of a port run never mentions it.
pub(crate) fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    *v == T::default()
}

fn is_zero(v: &usize) -> bool {
    *v == 0
}
fn is_default_bonus(v: &f64) -> bool {
    *v == FAMILY_BONUS
}
fn is_default_cons_income(v: &f64) -> bool {
    *v == FAMILY_CONS_INCOME
}
fn is_default_cons_wealth(v: &f64) -> bool {
    *v == FAMILY_CONS_WEALTH
}
fn is_default_years_purchase(v: &f64) -> bool {
    *v == YEARS_PURCHASE
}
fn is_default_rent_floor(v: &f64) -> bool {
    *v == LAND_RENT_FLOOR
}
fn is_default_start_share(v: &f64) -> bool {
    *v == FAMILY_START_SHARE
}
fn is_default_start_cash(v: &f64) -> bool {
    *v == FAMILY_START_CASH
}
fn is_default_reserve(v: &f64) -> bool {
    *v == FAMILY_RESERVE_MONTHS
}
fn is_default_max_share(v: &f64) -> bool {
    *v == FAMILY_MAX_SHARE
}
fn is_default_max_price_ratio(v: &f64) -> bool {
    *v == FAMILY_MAX_PRICE_RATIO
}
fn is_default_seat_bonus(v: &f64) -> bool {
    *v == FAMILY_SEAT_BONUS
}
fn is_default_birth_rate(v: &f64) -> bool {
    *v == FAMILY_BIRTH_RATE
}
fn is_default_absentee_cut(v: &f64) -> bool {
    *v == ABSENTEE_CUT
}
fn is_default_university_gain(v: &f64) -> bool {
    *v == UNIVERSITY_GAIN
}

pub fn default_params() -> Params {
    Params {
        seed: 7,
        W: 120,
        H: 84,
        n_towns: 6,
        pop0: 135_000,
        T_max: 48,
        F_max: 20_000,
        warmup_years: 10,
        // people
        participation: 0.5,
        growth_slope: 0.03,
        growth_base: -0.004,
        growth_cap: 0.011,
        migration_rate: 0.05,
        migration_dist: 60.0,
        // transport
        freight: 0.015,
        handling: 0.03,
        trade_k: 0.6,
        trade_cap: 0.3,
        road_thresholds: [1500.0, 9000.0, 40000.0],
        // markets
        logit_theta: 0.25,
        heat_theta: 0.2,
        price_speed: 0.03,
        price_flow: 0.08,
        labor_adjust_ex: 0.15,
        labor_adjust_firm: 0.35,
        price_target_months: 2.0,
        price_floor: 0.05,
        decay: vec![
            0.03, 0.01, 0.005, 0.002, 0.002, 0.002, 0.003, 0.01, 0.002, 0.002, 0.003, 0.005, 0.002, 0.003, 0.002, 0.5,
        ],
        price_ceiling: 40.0,
        // households
        cons_income: 0.9,
        cons_wealth: 0.035,
        lord_share: 0.01,
        lord_cons_income: 0.75,
        lord_cons_wealth: 0.04,
        urban_rent_rate: 0.12,
        tools_boost: 0.5,
        tools_scale: 0.01,
        // money and ownership
        money_growth: 0.02,
        rent_regime: RentRegime::Landlords, // or Commons: land rent paid out as an equal dividend
        firm_div_rate: 0.4,
        firm_buffer_months: 4.0,
        merchant_spread: 0.06,
        merch_div_rate: 0.15,
        merch_buffer_months: 2.0,
        // firms
        hurdle: 0.12,
        invest_frac_hh: 0.08,
        invest_frac_lord: 0.25,
        util_target_margin: 0.03,
        util_band: 0.04,
        depreciation: 0.04,
        entry_K_frac: 0.4,
        cluster_K_mult: 6.0,
        footprint_K_mult: 4.0,
        agglo_same: 0.08,
        candidates: 32,
        // mines
        pump_input: 0.0004,
        pump_boost: 2.0,
        wagon_input: 0.002,
        wagon_boost: 0.6,
        husbandry_boost: 0.6,
        farm_mach_boost: 0.8,
        farm_mach_input: 0.0004,
        // land market
        land_theta: 3.0,
        land_adjust: 0.2,
        land_fixed_cost: 0.15,
        expect_rate: 0.3,
        // forests
        forest_capacity: 16000.0,
        forest_regrow: 0.12,
        clearing_timber: 3000.0,
        // tech
        discovery_rate: 0.18,
        diffusion_rate: 0.3,
        diffusion_dist: 35.0,
        learn_cap: 4.0,
        // railways
        rail_rails_per_cell: 40.0,
        rail_timber_per_cell: 30.0,
        rail_stone_per_cell: 30.0,
        rail_hurdle: 0.06,
        rail_induced: 2.5,
        rail_max_projects: 6,
        rail_know: 0.25,
        spur_min_flow: 20000.0,
        // towns
        found_activity: 600.0,
        found_distance: 10.0,
        urban_density0: 500.0,
        urban_density_growth: 14.0,
        // detectors
        crossing_lo: 0.45,
        crossing_hi: 0.55,
        // families (off)
        n_families: 0,
        family_bonus: FAMILY_BONUS,
        family_cons_income: FAMILY_CONS_INCOME,
        family_cons_wealth: FAMILY_CONS_WEALTH,
        years_purchase: YEARS_PURCHASE,
        land_rent_floor: LAND_RENT_FLOOR,
        family_start_share: FAMILY_START_SHARE,
        family_start_cash: FAMILY_START_CASH,
        family_reserve_months: FAMILY_RESERVE_MONTHS,
        family_max_share: FAMILY_MAX_SHARE,
        family_max_price_ratio: FAMILY_MAX_PRICE_RATIO,
        family_seat_bonus: FAMILY_SEAT_BONUS,
        family_birth_rate: FAMILY_BIRTH_RATE,
        absentee_cut: ABSENTEE_CUT,
        university_gain: UNIVERSITY_GAIN,
        end_year: None,
        prices: crate::prices::PriceParams::default(),
        engel: crate::engel::EngelParams::default(),
        machines: crate::machines::MachineParams::default(),
        harvest: crate::harvest::HarvestParams::default(),
        chronicle: crate::chronicle::ChronicleParams::default(),
        works: WorksParams::default(),
        railways: crate::railways::RailwayParams::default(),
        carting: crate::carting::CartingParams::default(),
        people: crate::peopling::PeopleParams::default(),
        land: crate::landform::LandParams::default(),
    }
}

impl Default for Params {
    fn default() -> Params {
        default_params()
    }
}

/// The rules a live game is played under: the port's numbers, plus every mechanism
/// that is not in the Python switched on with its tuned values. Each new mechanism
/// adds its switch here (`P.canals = CanalParams::game();`) and nowhere in
/// `default_params`, which stays the port. Families are not switched on here: how
/// many there are is the caller's choice (`n_families`), as it is under the port.
pub fn game_params() -> Params {
    let mut P = default_params();
    P.prices = crate::prices::PriceParams::game();
    P.engel = crate::engel::EngelParams::game();
    P.machines = crate::machines::MachineParams::game();
    P.harvest = crate::harvest::HarvestParams::game();
    P.chronicle = crate::chronicle::ChronicleParams::game();
    P.works = WorksParams::game();
    P.railways = crate::railways::RailwayParams::game();
    P.carting = crate::carting::CartingParams::game();
    P.people = crate::peopling::PeopleParams::game();
    // land (docs/rules/land.md) is not switched on yet: the viewer cannot draw
    // it, and three tests are written to the port's island. When it is, the
    // line is `P.land = crate::landform::LandParams::game();`.
    P
}

/// Which rules a run is played under (`--rules port|game` on the command line).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rules {
    /// The port of the Python: `default_params` and `default_content`.
    Port,
    /// The game's rules: `game_params` and `game_content`.
    Game,
}

impl Rules {
    pub fn as_str(self) -> &'static str {
        match self {
            Rules::Port => "port",
            Rules::Game => "game",
        }
    }

    pub fn parse(s: &str) -> Option<Rules> {
        match s {
            "port" => Some(Rules::Port),
            "game" => Some(Rules::Game),
            _ => None,
        }
    }

    pub fn params(self) -> Params {
        match self {
            Rules::Port => default_params(),
            Rules::Game => game_params(),
        }
    }

    pub fn content(self) -> crate::content::Content {
        match self {
            Rules::Port => crate::content::default_content(),
            Rules::Game => crate::content::game_content(),
        }
    }
}

/// Lay `over` on `base`: objects are merged key by key, so a partial document for
/// one mechanism (`{"canals": {"cost": 0.2}}`) keeps the rest of its numbers;
/// anything else replaces what was there.
fn merge_json(base: &mut serde_json::Value, over: &serde_json::Value) {
    match (base, over) {
        (serde_json::Value::Object(b), serde_json::Value::Object(o)) => {
            for (k, v) in o {
                match b.get_mut(k) {
                    Some(slot) => merge_json(slot, v),
                    None => {
                        b.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        (b, o) => *b = o.clone(),
    }
}

impl Params {
    /// Defaults overridden by a partial TOML document, e.g. `seed = 3` and `land_theta = 2.5`.
    pub fn from_toml_str(text: &str) -> Result<Params, String> {
        toml::from_str(text).map_err(|e| format!("params: {e}"))
    }

    /// Defaults overridden by a partial JSON object, e.g. `{"seed": 3, "rent_regime": "commons"}`.
    pub fn from_json_str(text: &str) -> Result<Params, String> {
        serde_json::from_str(text).map_err(|e| format!("params: {e}"))
    }

    /// These parameters overridden by a partial JSON object. A mechanism's
    /// struct is merged field by field, not replaced.
    pub fn with_json_overrides(&self, text: &str) -> Result<Params, String> {
        let over: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("params: {e}"))?;
        self.with_overrides(&over)
    }

    /// These parameters overridden by a partial TOML document.
    pub fn with_toml_overrides(&self, text: &str) -> Result<Params, String> {
        let over: serde_json::Value = toml::from_str(text).map_err(|e| format!("params: {e}"))?;
        self.with_overrides(&over)
    }

    fn with_overrides(&self, over: &serde_json::Value) -> Result<Params, String> {
        if !over.is_object() {
            return Err("params: overrides must be a JSON object or a TOML table".to_string());
        }
        let mut base = serde_json::to_value(self).map_err(|e| format!("params: {e}"))?;
        merge_json(&mut base, over);
        serde_json::from_value(base).map_err(|e| format!("params: {e}"))
    }

    /// Load overrides from a `.toml` or `.json` file on top of the defaults.
    pub fn load(path: &std::path::Path) -> Result<Params, String> {
        default_params().load_over(path)
    }

    /// Load overrides from a `.toml` or `.json` file on top of these parameters
    /// (the game's rules, say).
    pub fn load_over(&self, path: &std::path::Path) -> Result<Params, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let read = match path.extension().and_then(|e| e.to_str()) {
            Some("json") => self.with_json_overrides(&text),
            _ => self.with_toml_overrides(&text),
        };
        // name the file: a wrong field is found after the merge, where its line is no longer known
        read.map_err(|e| format!("{}: {e}", path.display()))
    }
}
