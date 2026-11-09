//! Default parameters. Everything tunable lives here as one plain struct. Python: `params.py`.
//!
//! The field order is the order of Python's `default_params` dict, so `params.json`
//! comes out the same. Overrides are partial TOML or JSON documents: any field left
//! out keeps its default, and an unknown field is an error (Python would silently
//! store a misspelt key).

use serde::{Deserialize, Serialize};

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
    /// Share of its cash a family spends each month.
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
    /// AI families keep this many months of spending as a cash reserve.
    #[serde(skip_serializing_if = "is_default_reserve")]
    pub family_reserve_months: f64,
    /// AI families build only where the works' output is at most this share of the town's demand...
    #[serde(skip_serializing_if = "is_default_max_share")]
    pub family_max_share: f64,
    /// ...and the good's price is at most this many times its base price.
    #[serde(skip_serializing_if = "is_default_max_price_ratio")]
    pub family_max_price_ratio: f64,
    /// Last simulated year; `None` means the content's end year (Python: `P.get("end_year", C["end_year"])`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_year: Option<i64>,
}

const FAMILY_BONUS: f64 = 0.08;
const FAMILY_CONS_INCOME: f64 = 0.6;
const FAMILY_CONS_WEALTH: f64 = 0.02;
const YEARS_PURCHASE: f64 = 25.0;
const LAND_RENT_FLOOR: f64 = 0.5;
const FAMILY_START_SHARE: f64 = 0.03;
const FAMILY_START_CASH: f64 = 0.004;
const FAMILY_RESERVE_MONTHS: f64 = 12.0;
const FAMILY_MAX_SHARE: f64 = 0.25;
const FAMILY_MAX_PRICE_RATIO: f64 = 3.0;

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
        end_year: None,
    }
}

impl Default for Params {
    fn default() -> Params {
        default_params()
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

    /// These parameters overridden by a partial JSON object.
    pub fn with_json_overrides(&self, text: &str) -> Result<Params, String> {
        let mut base = serde_json::to_value(self).map_err(|e| format!("params: {e}"))?;
        let over: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("params: {e}"))?;
        let (Some(b), Some(o)) = (base.as_object_mut(), over.as_object()) else {
            return Err("params: overrides must be a JSON object".to_string());
        };
        for (k, v) in o {
            b.insert(k.clone(), v.clone());
        }
        serde_json::from_value(base).map_err(|e| format!("params: {e}"))
    }

    /// Load overrides from a `.toml` or `.json` file on top of the defaults.
    pub fn load(path: &std::path::Path) -> Result<Params, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        match path.extension().and_then(|e| e.to_str()) {
            Some("json") => Params::from_json_str(&text),
            _ => Params::from_toml_str(&text),
        }
    }
}
