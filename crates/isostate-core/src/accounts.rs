//! Books that say who makes, moves, buys and earns what. Not in the Python
//! (`docs/rules/accounts.md`).
//!
//! `econ::step_month` works out, every month, what each farm and works made,
//! what every buyer got, what went between the towns and who was paid for it,
//! and then keeps only a few totals. With `accounts.enabled` the rest is
//! gathered here: a year's books for every town and good, for every works, and
//! an income account for the island (what was paid for work, to the owners of
//! works, to merchants and to the owners of land, and who received it).
//!
//! It is an observer. Nothing in the economy reads `State::acct`, and a run
//! with the books on is the same run, number for number, as one with them off.
//! The books check themselves as they go: for every town and good, what was
//! made and what came in must equal what was bought, sent, spoiled and added
//! to stock; the incomes received must be the incomes paid; and the island's
//! income must be its spending. What is left over is kept in each year's
//! account (`IslandYear::balance_*`), so a leak in the economy shows as a
//! number, not as a silence.

use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::content::Content;
use crate::facts::Kind;
use crate::families;
use crate::mat::Mat;
use crate::observer::{Series, annual};
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::State;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AccountsParams {
    /// Keep the books.
    pub enabled: bool,
    /// Keep every works' books too (the larger part of what a recorded run
    /// carries). Off, only the towns' books and the island's account are kept.
    pub works: bool,
}

impl Default for AccountsParams {
    /// Off: what the port runs with.
    fn default() -> AccountsParams {
        AccountsParams { enabled: false, works: true }
    }
}

impl AccountsParams {
    /// The game's books.
    pub fn game() -> AccountsParams {
        AccountsParams { enabled: true, ..AccountsParams::default() }
    }
}

// ---------------------------------------------------------------- the tables' columns
//
// Each table is a flat array with one row of numbers per (town, good), (town,
// sector), (town, method), town or works. The names below are the columns, in
// order; they are written into the run file's header, and the viewer reads the
// tables by them.

/// Per town and good, `[(town * G + good) * KG + column]`. Quantities are the
/// year's totals in the good's units.
pub const GOODS_FIELDS: [&str; 22] = [
    "made",         // by the town's land and works, timber from clearing included
    "cleared",      // of that, timber from woodland cleared at the year's end
    "households",   // bought by households
    "landlords",    // bought by the town's landlords for themselves
    "families",     // bought by families seated here (through the landlords)
    "works_inputs", // used up by works as materials
    "works_fuel",   // burnt by works
    "land",         // bought by farms, pits and mines (machinery, rails)
    "building",     // bought by works being built or enlarged
    "projects",     // bought by railways under construction
    "sent",         // carried to other towns
    "received",     // arrived from other towns
    "spoiled",      // lost in store
    "moved",        // stock that came (+) or went (-) with districts that changed market
    "oversold",     // sold though it was not in store (see "What does not balance" in the page)
    "stock_open",   // in the merchants' hands as the year began
    "stock",        // and as it ended
    "asked",        // everything buyers asked for; less what was bought, the unmet requests
    "price",        // mean of the twelve month-end prices
    "price_low",
    "price_high",
    "anchor",       // mean of the merchants' anchor (prices mechanism), 0 when it is off
];
pub const KG: usize = GOODS_FIELDS.len();
const G_MADE: usize = 0;
const G_CLEARED: usize = 1;
const G_HH: usize = 2;
const G_LORDS: usize = 3;
const G_FAM: usize = 4;
const G_INPUTS: usize = 5;
const G_FUEL: usize = 6;
const G_LAND: usize = 7;
const G_BUILD: usize = 8;
const G_PROJ: usize = 9;
const G_SENT: usize = 10;
const G_RECV: usize = 11;
const G_SPOILED: usize = 12;
const G_MOVED: usize = 13;
const G_OVERSOLD: usize = 14;
const G_OPEN: usize = 15;
const G_STOCK: usize = 16;
const G_ASKED: usize = 17;
const G_PRICE: usize = 18;
const G_LOW: usize = 19;
const G_HIGH: usize = 20;
const G_ANCHOR: usize = 21;

/// Per town and land sector, `[(town * NS + sector) * KL + column]`. The
/// merchants pay `value * (1 - merchant_spread)` for the produce, and that is
/// wages + carting + inputs + rent.
pub const LAND_FIELDS: [&str; 7] = [
    "out",     // quantity made
    "value",   // at the town's price
    "hands",   // mean over the year
    "wages",
    "carting", // paid to carry the produce to market
    "inputs",  // machinery and rails bought
    "rent",    // what is left, to the owner of the land
];
pub const KL: usize = LAND_FIELDS.len();
const L_OUT: usize = 0;
const L_VALUE: usize = 1;
const L_HANDS: usize = 2;
const L_WAGES: usize = 3;
const L_CART: usize = 4;
const L_INPUTS: usize = 5;
const L_RENT: usize = 6;

/// Per town and method, `[(town * M + method) * KM + column]`. The merchants
/// pay `value * (1 - merchant_spread)` for the output; of that `cart_out` goes
/// to the carters and the rest is the works' revenue, which is inputs + fuel +
/// cart_in + wages + rent + managers + profit.
pub const METHOD_FIELDS: [&str; 14] = [
    "out",       // quantity made
    "value",     // at the town's price
    "works",     // works at work, mean over the year
    "capacity",  // their capacity a month, mean over the year
    "hands",     // mean over the year
    "wages",
    "inputs",    // materials, at the town's prices
    "fuel",      // fuel, at the town's prices
    "cart_in",   // carting of materials and fuel out to the works
    "cart_out",  // carting of the output in to the market
    "rent",      // for the works' sites
    "managers",  // hired managers of family works with no branch over them
    "profit",    // after the managers
    "dividends", // paid out of it, to households or to the owning family
];
pub const KM: usize = METHOD_FIELDS.len();
const M_OUT: usize = 0;
const M_VALUE: usize = 1;
const M_WORKS: usize = 2;
const M_CAP: usize = 3;
const M_HANDS: usize = 4;
const M_WAGES: usize = 5;
const M_INPUTS: usize = 6;
const M_FUEL: usize = 7;
const M_CART_IN: usize = 8;
const M_CART_OUT: usize = 9;
const M_RENT: usize = 10;
const M_MANAGERS: usize = 11;
const M_PROFIT: usize = 12;
const M_DIV: usize = 13;

/// Per town, `[town * KT + column]`. Money is the year's total.
pub const TOWN_FIELDS: [&str; 32] = [
    "pop",             // at the year's end
    "wage",            // mean of the twelve months
    "supply",          // hands for hire (carters already taken out), mean
    "carters",         // hands carting, mean (0 unless carting takes labour)
    "hands_land",      // mean
    "hands_works",     // mean
    // what the town's households received, by kind
    "pay_works",
    "pay_land",
    "pay_projects",    // navvies on the railways
    "cart_works",      // carting for works, in and out
    "cart_land",       // carting of the land's produce
    "cart_trade",      // carting of what the town sent to other towns
    "managers",
    "div_works",       // dividends of works that no family owns
    "div_merchants",
    "fund",            // the common fund's dividend (rents, under the commons regime)
    "issue",           // new money
    // what went to owners who are not the town's households
    "div_families",    // dividends of family works here
    "rent_land",       // rents of farms, woods, pits and mines in the town's district
    "rent_sites",      // rents of works' sites
    "rent_town",       // rents of town houses, paid by households
    "rent_families",   // of those three, what went to families
    // spending
    "spend_households", // on goods
    "spend_landlords",  // on goods, the landlords' own
    "spend_families",   // on goods, families seated here
    "invest",           // works' purchases of capital goods
    "projects",         // railways' outlay: materials and navvies
    // merchants and money
    "merchants_net",    // the merchants' takings less what they paid out, before their dividend
    "stock_value",      // their stock at the year's end, at that month's prices
    "cash_households",  // at the year's end
    "cash_landlords",
    "cash_merchants",
];
pub const KT: usize = TOWN_FIELDS.len();
const T_POP: usize = 0;
const T_WAGE: usize = 1;
const T_SUPPLY: usize = 2;
const T_CARTERS: usize = 3;
const T_HANDS_LAND: usize = 4;
const T_HANDS_WORKS: usize = 5;
const T_PAY_WORKS: usize = 6;
const T_PAY_LAND: usize = 7;
const T_PAY_PROJ: usize = 8;
const T_CART_WORKS: usize = 9;
const T_CART_LAND: usize = 10;
const T_CART_TRADE: usize = 11;
const T_MANAGERS: usize = 12;
const T_DIV_WORKS: usize = 13;
const T_DIV_MERCH: usize = 14;
const T_FUND: usize = 15;
const T_ISSUE: usize = 16;
const T_DIV_FAM: usize = 17;
const T_RENT_LAND: usize = 18;
const T_RENT_SITES: usize = 19;
const T_RENT_TOWN: usize = 20;
const T_RENT_FAM: usize = 21;
const T_SPEND_HH: usize = 22;
const T_SPEND_LORDS: usize = 23;
const T_SPEND_FAM: usize = 24;
const T_INVEST: usize = 25;
const T_PROJECTS: usize = 26;
const T_MERCH_NET: usize = 27;
const T_STOCK_VALUE: usize = 28;
const T_CASH_HH: usize = 29;
const T_CASH_LORDS: usize = 30;
const T_CASH_MERCH: usize = 31;
/// The household kinds, `T_PAY_WORKS..=T_ISSUE`: they add up to what the town's households received.
const HH_KINDS: std::ops::RangeInclusive<usize> = T_PAY_WORKS..=T_ISSUE;

/// Per works and year, `[row * KW + column]`.
pub const WORKS_FIELDS: [&str; 21] = [
    "owner",     // the owning family, -1 for none
    "method",
    "town",
    "capacity",  // a month, at the year's end
    "out",       // quantity made
    "value",     // at the town's price
    "revenue",   // what the merchants paid the works: value less their spread and the carting in to market
    "inputs",    // materials, at the town's prices
    "fuel",      // fuel, at the town's prices
    "cart_in",   // carting of both out to the works
    "cart_out",  // carting of the output to market (paid by the merchants, out of the price)
    "hands",     // mean over the year
    "wages",
    "rent",
    "managers",
    "profit",    // revenue - inputs - fuel - cart_in - wages - rent - managers
    "dividends",
    "invest",    // paid for capital goods
    "util",      // output over capacity, mean of the months it worked
    "cash",      // at the year's end
    "months",    // months it worked
];
pub const KW: usize = WORKS_FIELDS.len();
const W_OWNER: usize = 0;
const W_METHOD: usize = 1;
const W_TOWN: usize = 2;
const W_CAP: usize = 3;
const W_OUT: usize = 4;
const W_VALUE: usize = 5;
const W_REVENUE: usize = 6;
const W_INPUTS: usize = 7;
const W_FUEL: usize = 8;
const W_CART_IN: usize = 9;
const W_CART_OUT: usize = 10;
const W_HANDS: usize = 11;
const W_WAGES: usize = 12;
const W_RENT: usize = 13;
const W_MANAGERS: usize = 14;
const W_PROFIT: usize = 15;
const W_DIV: usize = 16;
const W_INVEST: usize = 17;
const W_UTIL: usize = 18;
const W_CASH: usize = 19;
const W_MONTHS: usize = 20;

/// Per month of the last complete year, town and good, `[((month * T + town) * G + good) * KMO + column]`.
pub const MONTH_FIELDS: [&str; 4] = ["price", "made", "bought", "stock"];
pub const KMO: usize = MONTH_FIELDS.len();

// ---------------------------------------------------------------- what is kept

/// What one family received in a year.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct FamilyIncome {
    pub dividends: f64,
    pub rents: f64,
}

/// Who holds the money at the year's end (the purses `econ::money_total` counts).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Money {
    pub households: f64,
    pub landlords: f64,
    pub merchants: f64,
    pub works: f64,
    pub families: f64,
    pub projects: f64,
    pub fund: f64,
}

/// The island's income account for one year. Money is the year's total.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct IslandYear {
    pub year: i64,
    // ---- paid for work
    pub pay_works: f64,
    pub pay_land: f64,
    /// Navvies on the railways.
    pub pay_projects: f64,
    /// Carters' pay: produce to market, works' materials and output, trade between towns.
    pub carting: f64,
    /// Hired managers of family works with no branch over them.
    pub managers: f64,
    // ---- to the owners of works
    /// Dividends of works no family owns, which go to their town's households.
    pub div_households: f64,
    pub div_families: f64,
    /// Profit the works kept (negative in a year of losses).
    pub works_kept: f64,
    // ---- to merchants
    /// The merchants' dividend, which goes to their town's households.
    pub merchants_div: f64,
    /// What the merchants took in less what they paid out and their dividend:
    /// cash, so a year of stock-building shows as a loss.
    pub merchants_kept: f64,
    // ---- to the owners of land
    /// Farms, pastures and woods.
    pub rent_farm: f64,
    /// Quarries, pits and mines.
    pub rent_pits: f64,
    /// Works' sites.
    pub rent_sites: f64,
    /// Town houses.
    pub rent_town: f64,
    /// Every land sector's rent, in the content's order.
    pub rent_by_sector: Vec<f64>,
    // ---- transfers
    /// New money, paid to households by head.
    pub issue: f64,
    /// The common fund's dividend to households (rents, under the commons regime).
    pub fund_div: f64,
    // ---- who received it
    /// Pay, dividends, the fund's dividend and new money.
    pub to_households: f64,
    pub to_landlords: f64,
    pub to_families: Vec<FamilyIncome>,
    pub to_fund: f64,
    // ---- spending
    pub spend_households: f64,
    pub spend_landlords: f64,
    pub spend_families: f64,
    /// Households' spending on each need, in the content's order (goods only;
    /// their house rent is `rent_town`).
    pub need_households: Vec<f64>,
    /// Landlords' and families' spending on each need.
    pub need_landlords: Vec<f64>,
    /// Works' purchases of capital goods.
    pub invest: f64,
    /// Railways' outlay: materials and navvies.
    pub projects: f64,
    // ---- the headlines
    /// Everything earned: pay, profits (works' and merchants') and rents.
    pub income: f64,
    /// Everything spent on final goods, house rent and building. It equals `income`.
    pub spending: f64,
    /// The four shares of `income`; they add up to 1.
    pub share_work: f64,
    pub share_works: f64,
    pub share_merchants: f64,
    pub share_land: f64,
    /// The share of what households received that was pay for work.
    pub pay_share: f64,
    /// Pay for work in the year: wages, carters, managers.
    pub wage_bill: f64,
    // ---- who owns what, at the year's end
    /// All works at depreciated capital cost (`families::works_value`).
    pub works_value: f64,
    pub works_value_families: f64,
    /// All land at `years_purchase` times the year's rents.
    pub land_value: f64,
    pub land_value_families: f64,
    pub money: Money,
    /// Works and land in years of the wage bill.
    pub works_years: f64,
    pub land_years: f64,
    pub ownership_years: f64,
    // ---- the books' own checks
    /// The worst town and good: |made + received + moved + oversold - bought -
    /// sent - spoiled - added to stock| over what passed through (opening
    /// stock, made and received).
    pub balance_goods: f64,
    /// Where: (town, good).
    pub balance_goods_at: (usize, usize),
    /// The same, in the good's units.
    pub balance_goods_units: f64,
    /// The worst month: incomes paid against incomes received, as a share.
    pub balance_income: f64,
    /// |income - spending| / spending.
    pub balance_spending: f64,
    /// Goods sold though they were not in store, at the year's mean prices, as
    /// a share of everything bought: the one place the economy itself leaks.
    pub oversold_share: f64,
}

/// One flow between two towns in a year.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Flow {
    pub from: u16,
    pub to: u16,
    pub good: u16,
    pub qty: f32,
    /// At the sending town's price.
    pub value: f32,
    /// The carting paid on it, to the sending town's carters.
    pub carting: f32,
}

/// What one method in one town used of one good in a year.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Use {
    pub town: u16,
    pub method: u16,
    pub good: u16,
    /// Used as material.
    pub used: f32,
    /// Burnt as fuel.
    pub burnt: f32,
    /// What both cost delivered to the works (the town's price and the carting out).
    pub cost: f32,
}

/// One complete year's books, as they are kept (single precision).
#[derive(Clone, Debug, Default)]
pub struct YearRec {
    /// `[T][G][KG]`
    pub goods: Vec<f32>,
    /// `[T][NS][KL]`
    pub land: Vec<f32>,
    /// `[T][M][KM]`
    pub methods: Vec<f32>,
    /// `[T][KT]`
    pub towns: Vec<f32>,
    /// `[T][S][2]`: households', then landlords' and families', spending on each need.
    pub needs: Vec<f32>,
    /// Every flow between towns, by (from, to, good).
    pub trade: Vec<Flow>,
    /// What each method in each town used of each good, by (town, method, good).
    pub uses: Vec<Use>,
    /// The works that worked, built or stood this year, by firm id, and their rows `[n][KW]`.
    pub works_id: Vec<u32>,
    pub works: Vec<f32>,
}

/// The year now running, in double precision.
#[derive(Clone, Debug, Default)]
struct Acc {
    goods: Vec<f64>,
    land: Vec<f64>,
    methods: Vec<f64>,
    towns: Vec<f64>,
    needs: Vec<f64>,
    /// `[(from * T + to) * G + good][3]`: quantity, value, carting.
    trade: Vec<f64>,
    /// `[(town * M + method) * G + good][3]`: used as material, burnt, cost at the works.
    uses: Vec<f64>,
    /// `[firm][KW]`
    works: Vec<f64>,
    /// `[firm][G][3]`: materials used, fuel burnt, and what both cost at the works.
    works_in: Vec<f64>,
    /// `[month][T][G][KMO]`
    months: Vec<f64>,
    month: usize,
    fam: Vec<FamilyIncome>,
    rent_sector: Vec<f64>,
    fund_in: f64,
    lords_in: f64,
    hh_in: f64,
    fund_div: f64,
    issue: f64,
    /// The worst month's income check.
    income_err: f64,
}

/// `State::acct`: empty while the mechanism is off.
#[derive(Clone, Debug, Default)]
pub struct Books {
    /// Set by `init` when `accounts.enabled`; every hook returns at once without it.
    pub on: bool,
    pub start_year: i64,
    dims: (usize, usize, usize, usize, usize),
    cur: Acc,
    /// The towns' stocks as the books last saw them.
    stock: Mat,
    /// The month's scratch: households' income by kind `[T][11]`, the works'
    /// dividends before the families took theirs, the fund as the month began.
    mk: Vec<f64>,
    /// The month's land table `[T][NS][5]`: out, hands, carting, machinery and rails got.
    ml: Vec<f64>,
    /// The goods each method can use: its materials, and every fuel if it needs heat.
    mgoods: Vec<Vec<usize>>,
    div: Vec<f64>,
    fund_open: f64,
    /// The month's rents, and of them what went to families.
    m_rents: f64,
    m_fam_rents: f64,
    /// The island's account of every complete year.
    pub years: Vec<IslandYear>,
    /// Every complete year's books.
    pub hist: Vec<YearRec>,
    /// The last complete year only: its months `[12][T][G][KMO]`, and each
    /// works' materials, fuel and their cost by good `[firm][G][3]`.
    pub last_months: Vec<f32>,
    pub last_works_in: Vec<f32>,
    /// Seconds spent keeping the books (wall time; nothing reads it but the report).
    pub seconds: f64,
}

/// What `econ::step_month` hands over once the month's goods have moved and
/// been paid for, before prices respond. Every field is a local of that
/// function, named as it is there.
pub struct Month<'a> {
    // land units
    pub ex: &'a [usize],
    pub et: &'a [usize],
    pub sec: &'a [usize],
    pub Y: &'a [f64],
    pub Lex: &'a [f64],
    pub cart_ex: &'a [f64],
    pub rent_ex: &'a [f64],
    pub mach_got: &'a [f64],
    pub rail_got: &'a [f64],
    // works
    pub fi: &'a [usize],
    pub ft: &'a [usize],
    pub fm: &'a [usize],
    pub fc: &'a [usize],
    pub out: &'a [usize],
    pub K: &'a [f64],
    pub q: &'a [f64],
    pub labor_f: &'a [f64],
    pub wages_f: &'a [f64],
    pub rent_f: &'a [f64],
    pub pay_in: &'a [f64],
    pub cart_in: &'a [f64],
    pub cart_out: &'a [f64],
    pub rev: &'a [f64],
    /// The hired managers' share of each works' profit (None without families).
    pub cut: Option<&'a [f64]>,
    pub a: &'a Mat,
    pub use_f: &'a Mat,
    pub pin: &'a Mat,
    // buyers
    pub hh_got: &'a Mat,
    pub lord_got: &'a Mat,
    pub fam_by_town: &'a [f64],
    pub lord_budget: &'a [f64],
    pub hh_spend: &'a [f64],
    pub lord_spend: &'a [f64],
    pub urban_pay: &'a [f64],
    pub q_hh: &'a Mat,
    pub q_l: &'a Mat,
    pub share: &'a Mat,
    pub fill: &'a Mat,
    pub lord_pop: &'a [f64],
    pub oi: &'a [usize],
    pub ot: &'a [usize],
    pub inv_got: &'a Mat,
    pub proj_got: &'a [Vec<f64>],
    // markets
    pub R: &'a Mat,
    pub sold: &'a Mat,
    pub prod_TG: &'a Mat,
    pub exports: &'a Mat,
    pub imports: &'a Mat,
    pub w: &'a [f64],
    pub supply: &'a [f64],
}

// ---------------------------------------------------------------- the hooks

fn new_acc(dims: (usize, usize, usize, usize, usize), F: usize, n_fam: usize, works: bool) -> Acc {
    let (T, G, NS, M, S) = dims;
    let mut goods = vec![0.0; T * G * KG];
    for k in 0..T * G {
        goods[k * KG + G_LOW] = f64::INFINITY;
        goods[k * KG + G_HIGH] = f64::NEG_INFINITY;
    }
    Acc {
        goods,
        land: vec![0.0; T * NS * KL],
        methods: vec![0.0; T * M * KM],
        towns: vec![0.0; T * KT],
        needs: vec![0.0; T * S * 2],
        trade: vec![0.0; T * T * G * 3],
        uses: vec![0.0; T * M * G * 3],
        works: if works { vec![0.0; F * KW] } else { Vec::new() },
        works_in: if works { vec![0.0; F * G * 3] } else { Vec::new() },
        months: vec![0.0; 12 * T * G * KMO],
        month: 0,
        fam: vec![FamilyIncome::default(); n_fam],
        rent_sector: vec![0.0; NS],
        ..Acc::default()
    }
}

/// Open the books on the warmed-up economy (`sim::start_from`, after the
/// families have their start). Does nothing unless `accounts.enabled`.
pub fn init(st: &mut State, C: &Content, P: &Params) {
    if !P.accounts.enabled {
        return;
    }
    let dims = (st.T_max, C.G, C.NS, C.M, C.S);
    let mut cur = new_acc(dims, st.f_alive.len(), st.families.len(), P.accounts.works);
    open_stock(&mut cur, &st.inv);
    st.acct = Books {
        on: true,
        start_year: C.start_year,
        dims,
        cur,
        stock: st.inv.clone(),
        mk: vec![0.0; st.T_max * HH_KINDS.clone().count()],
        ml: vec![0.0; st.T_max * C.NS * 5],
        mgoods: (0..C.M).map(|m| (0..C.G).filter(|&g| C.m_in[(m, g)] > 0.0 || (C.m_heat[m] > 0.0 && C.fuel_goods.contains(&g))).collect()).collect(),
        ..Books::default()
    };
}

/// Wipe the year's tables for the next one. Works are only ever added, so the
/// first `F` rows of the works' tables are all that can have been written.
fn reset(a: &mut Acc, dims: (usize, usize, usize, usize, usize), F: usize, n_fam: usize) {
    let (T, G, ..) = dims;
    for v in [&mut a.goods, &mut a.land, &mut a.methods, &mut a.towns, &mut a.needs, &mut a.trade, &mut a.uses, &mut a.months, &mut a.rent_sector] {
        v.fill(0.0);
    }
    for k in 0..T * G {
        a.goods[k * KG + G_LOW] = f64::INFINITY;
        a.goods[k * KG + G_HIGH] = f64::NEG_INFINITY;
    }
    let rows = F.min(a.works.len() / KW);
    a.works[..rows * KW].fill(0.0);
    a.works_in[..rows * G * 3].fill(0.0);
    a.fam = vec![FamilyIncome::default(); n_fam];
    a.month = 0;
    (a.fund_in, a.lords_in, a.hh_in, a.fund_div, a.issue, a.income_err) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
}

fn open_stock(a: &mut Acc, inv: &Mat) {
    for (k, &v) in inv.data.iter().enumerate() {
        a.goods[k * KG + G_OPEN] = v;
    }
}

/// Whatever the towns' stocks did since the books last saw them came or went
/// with districts that changed market (`people::resettle_new_towns`,
/// `peopling::move_with_cells`): between two months nothing else touches them
/// but clearing, which `note_cleared` has already counted.
fn sync_stock(b: &mut Books, inv: &Mat) {
    for (k, (&now, was)) in inv.data.iter().zip(b.stock.data.iter_mut()).enumerate() {
        if now != *was {
            b.cur.goods[k * KG + G_MOVED] += now - *was;
            *was = now;
        }
    }
}

/// The first thing in a month.
pub fn open_month(st: &mut State) {
    if !st.acct.on {
        return;
    }
    let t0 = Instant::now();
    let mut b = std::mem::take(&mut st.acct);
    sync_stock(&mut b, &st.inv);
    b.mk.iter_mut().for_each(|v| *v = 0.0);
    b.div.clear();
    b.fund_open = st.fund_cash;
    (b.m_rents, b.m_fam_rents) = (0.0, 0.0);
    b.seconds += t0.elapsed().as_secs_f64();
    st.acct = b;
}

/// Timber from woodland cleared at the year's end (`land::update_land_use`),
/// which goes straight into a town's stock.
pub fn note_cleared(st: &mut State, t: usize, g: usize, qty: f64) {
    if !st.acct.on {
        return;
    }
    let G = st.acct.dims.1;
    let k = (t * G + g) * KG;
    st.acct.cur.goods[k + G_MADE] += qty;
    st.acct.cur.goods[k + G_CLEARED] += qty;
    st.acct.stock[(t, g)] += qty;
}

/// The month's trade between towns, from inside the trade block: `qt` and `tau`
/// are `[(i * na + j) * G + g]` over the active towns `A_idx`, `p` the prices
/// the goods changed hands at.
pub fn note_trade(b: &mut Books, A_idx: &[usize], qt: &[f64], tau: &[f64], p: &Mat) {
    if !b.on {
        return;
    }
    let t0 = Instant::now();
    let (T, G, ..) = b.dims;
    let na = A_idx.len();
    let kinds = HH_KINDS.clone().count();
    for (i, &ti) in A_idx.iter().enumerate() {
        for (j, &tj) in A_idx.iter().enumerate() {
            let base = (i * na + j) * G;
            for g in 0..G {
                let v = qt[base + g];
                if v > 0.0 {
                    let k = ((ti * T + tj) * G + g) * 3;
                    let cart = v * tau[base + g];
                    b.cur.trade[k] += v;
                    b.cur.trade[k + 1] += v * p[(ti, g)];
                    b.cur.trade[k + 2] += cart;
                    b.mk[ti * kinds + (T_CART_TRADE - T_PAY_WORKS)] += cart;
                }
            }
        }
    }
    b.seconds += t0.elapsed().as_secs_f64();
}

/// The month's goods and what was paid for them (`econ::step_month`, after the
/// trade between towns and before prices respond).
pub fn note_month(st: &mut State, C: &Content, P: &Params, m: &Month) {
    if !st.acct.on {
        return;
    }
    let t0 = Instant::now();
    let mut b = std::mem::take(&mut st.acct);
    let (T, G, NS, M, S) = b.dims;
    let kinds = HH_KINDS.clone().count();
    let hk = |k: usize| k - T_PAY_WORKS;
    let p = &st.price;
    let (gm, gr) = (C.gi("machinery"), C.gi("rails"));
    let fam_on = families::active(st);
    let a = &mut b.cur;
    let mk = &mut b.mk;
    let (mut rents, mut fam_rents) = (0.0, 0.0);

    // ---- the land. There are tens of thousands of units and each of the month's
    // arrays over them is a quarter of a megabyte, so as few as will do are read:
    // what each unit made, its hands, its carting and the machines it got, added
    // up by town and sector. A sector's produce sells at one price in a town and
    // its hands are paid one wage, so the rest follows from the sums exactly as
    // econ::step_month works it out unit by unit: value = price * out, wages =
    // wage * hands, inputs at the town's prices, and rent is what is left of
    // value * (1 - spread). (The month's income check holds this to what the
    // landlords were in fact paid.)
    const ML: usize = 5;
    let ml = &mut b.ml;
    ml.fill(0.0);
    for u in 0..m.et.len() {
        let r = &mut ml[(m.et[u] * NS + m.sec[u]) * ML..][..ML];
        r[0] += m.Y[u];
        r[1] += m.Lex[u];
        r[2] += m.cart_ex[u];
        r[3] += m.mach_got[u];
        r[4] += m.rail_got[u];
    }
    let spread = P.merchant_spread;
    for t in (0..T).filter(|&t| st.town_active[t]) {
        for s in 0..NS {
            let r = &ml[(t * NS + s) * ML..][..ML];
            let (out, hands, cart) = (r[0], r[1], r[2]);
            let value = p[(t, C.sector_good[s])] * out;
            let wages = m.w[t] * hands;
            let inputs = r[3] * p[(t, gm)] + r[4] * p[(t, gr)];
            let rent = value * (1.0 - spread) - cart - wages - inputs;
            let l = &mut a.land[(t * NS + s) * KL..][..KL];
            l[L_OUT] += out;
            l[L_VALUE] += value;
            l[L_HANDS] += hands;
            l[L_WAGES] += wages;
            l[L_CART] += cart;
            l[L_INPUTS] += inputs;
            l[L_RENT] += rent;
            a.rent_sector[s] += rent;
            a.goods[(t * G + gm) * KG + G_LAND] += r[3];
            a.goods[(t * G + gr) * KG + G_LAND] += r[4];
            a.towns[t * KT + T_HANDS_LAND] += hands;
            a.towns[t * KT + T_RENT_LAND] += rent;
            rents += rent;
            mk[t * kinds + hk(T_PAY_LAND)] += wages;
            mk[t * kinds + hk(T_CART_LAND)] += cart;
        }
    }

    // ---- the works
    let keep_works = !a.works.is_empty();
    for i in 0..m.fi.len() {
        let (f, t, me, g_out) = (m.fi[i], m.ft[i], m.fm[i], m.out[i]);
        let gross = m.rev[i] - m.pay_in[i] - m.wages_f[i] - m.rent_f[i];
        // a family works with no branch over it pays a hired manager out of its profit
        let manager = match m.cut {
            Some(c) if c[i] > 0.0 && gross > 0.0 => c[i] * gross,
            _ => 0.0,
        };
        // materials and fuel at what the merchants were paid for them; the rest of
        // what the works paid is the carting out to it
        let (mut mat_cost, mut all_cost) = (0.0, 0.0);
        for &g in &b.mgoods[me] {
            let used = m.use_f[(i, g)];
            if used > 0.0 {
                let mat = m.q[i] * m.a[(i, g)];
                let at_town = m.pin[(i, g)].min(p[(t, g)]);
                mat_cost += mat * at_town;
                all_cost += used * at_town;
                let k = (t * G + g) * KG;
                a.goods[k + G_INPUTS] += mat;
                a.goods[k + G_FUEL] += used - mat;
                let ku = ((t * M + me) * G + g) * 3;
                a.uses[ku] += mat;
                a.uses[ku + 1] += used - mat;
                a.uses[ku + 2] += used * m.pin[(i, g)];
                if keep_works {
                    let kw = (f * G + g) * 3;
                    a.works_in[kw] += mat;
                    a.works_in[kw + 1] += used - mat;
                    a.works_in[kw + 2] += used * m.pin[(i, g)];
                }
            }
        }
        let value = m.q[i] * p[(t, g_out)];
        let k = (t * M + me) * KM;
        a.methods[k + M_OUT] += m.q[i];
        a.methods[k + M_VALUE] += value;
        a.methods[k + M_WORKS] += 1.0;
        a.methods[k + M_CAP] += m.K[i];
        a.methods[k + M_HANDS] += m.labor_f[i];
        a.methods[k + M_WAGES] += m.wages_f[i];
        a.methods[k + M_INPUTS] += mat_cost;
        a.methods[k + M_FUEL] += all_cost - mat_cost;
        a.methods[k + M_CART_IN] += m.cart_in[i];
        a.methods[k + M_CART_OUT] += m.cart_out[i];
        a.methods[k + M_RENT] += m.rent_f[i];
        a.methods[k + M_MANAGERS] += manager;
        a.methods[k + M_PROFIT] += gross - manager;
        a.towns[t * KT + T_HANDS_WORKS] += m.labor_f[i];
        a.towns[t * KT + T_RENT_SITES] += m.rent_f[i];
        rents += m.rent_f[i];
        mk[t * kinds + hk(T_PAY_WORKS)] += m.wages_f[i];
        mk[t * kinds + hk(T_CART_WORKS)] += m.cart_in[i] + m.cart_out[i];
        mk[t * kinds + hk(T_MANAGERS)] += manager;
        if fam_on {
            if let Some(o) = families::owner_of_cell(st, m.fc[i]) {
                a.fam[o].rents += m.rent_f[i];
                a.towns[t * KT + T_RENT_FAM] += m.rent_f[i];
                fam_rents += m.rent_f[i];
            }
        }
        if keep_works {
            let w = &mut a.works[f * KW..(f + 1) * KW];
            w[W_OUT] += m.q[i];
            w[W_VALUE] += value;
            w[W_REVENUE] += m.rev[i];
            w[W_INPUTS] += mat_cost;
            w[W_FUEL] += all_cost - mat_cost;
            w[W_CART_IN] += m.cart_in[i];
            w[W_CART_OUT] += m.cart_out[i];
            w[W_HANDS] += m.labor_f[i];
            w[W_WAGES] += m.wages_f[i];
            w[W_RENT] += m.rent_f[i];
            w[W_MANAGERS] += manager;
            w[W_PROFIT] += gross - manager;
            w[W_UTIL] += m.q[i] / m.K[i].max(1e-12);
            w[W_MONTHS] += 1.0;
        }
    }

    // ---- works being built
    for (r, &f) in m.oi.iter().enumerate() {
        let t = m.ot[r];
        let mut pay = 0.0;
        for g in 0..G {
            let got = m.inv_got[(r, g)];
            if got != 0.0 {
                a.goods[(t * G + g) * KG + G_BUILD] += got;
                pay += got * p[(t, g)];
            }
        }
        a.towns[t * KT + T_INVEST] += pay;
        if keep_works {
            a.works[f * KW + W_INVEST] += pay;
        }
    }

    // ---- railways
    for (pr, got) in st.projects.iter().zip(m.proj_got) {
        let t = pr.town;
        let mut pay = 0.0;
        for g in 0..G {
            if got[g] != 0.0 {
                a.goods[(t * G + g) * KG + G_PROJ] += got[g];
                pay += got[g] * p[(t, g)];
            }
        }
        // econ::step_month: the navvies are paid three tenths of the materials
        let wages = 0.3 * pay;
        a.towns[t * KT + T_PROJECTS] += pay + wages;
        mk[t * kinds + hk(T_PAY_PROJ)] += wages;
    }

    // ---- households, landlords and families; the markets
    for t in (0..T).filter(|&t| st.town_active[t]) {
        // families spend through their seat's landlords: their part of what the
        // landlords bought is their part of the budget (`families::pay_spending`)
        let fam_share = if m.lord_budget[t] > 0.0 { (m.fam_by_town[t] / m.lord_budget[t]).clamp(0.0, 1.0) } else { 0.0 };
        let kt = t * KT;
        a.towns[kt + T_SPEND_HH] += m.hh_spend[t];
        a.towns[kt + T_SPEND_FAM] += m.lord_spend[t] * fam_share;
        a.towns[kt + T_SPEND_LORDS] += m.lord_spend[t] * (1.0 - fam_share);
        a.towns[kt + T_RENT_TOWN] += m.urban_pay[t];
        rents += m.urban_pay[t];
        a.towns[kt + T_WAGE] += m.w[t];
        a.towns[kt + T_SUPPLY] += m.supply[t];
        if st.carters.len() == T {
            a.towns[kt + T_CARTERS] += st.carters[t];
        }
        for g in 0..G {
            let k = (t * G + g) * KG;
            let lord = m.lord_got[(t, g)];
            a.goods[k + G_HH] += m.hh_got[(t, g)];
            a.goods[k + G_FAM] += lord * fam_share;
            a.goods[k + G_LORDS] += lord * (1.0 - fam_share);
            a.goods[k + G_MADE] += m.prod_TG[(t, g)];
            a.goods[k + G_SENT] += m.exports[(t, g)];
            a.goods[k + G_RECV] += m.imports[(t, g)];
            a.goods[k + G_ASKED] += m.R[(t, g)];
            // the stock, step by step as econ::step_month moves it: sales out, what
            // is left decays, the month's make in, never below nothing; then the
            // trade, and never below nothing again. Where more was sold than was
            // in store the floor makes up the difference: that is `oversold`.
            let sold = m.sold[(t, g)];
            let left = b.stock[(t, g)] - sold;
            let v1 = left * (1.0 - P.decay[g]) + m.prod_TG[(t, g)];
            let v2 = v1.max(0.0) + m.imports[(t, g)] - m.exports[(t, g)];
            a.goods[k + G_SPOILED] += left.max(0.0) * P.decay[g];
            a.goods[k + G_OVERSOLD] += (-left).max(0.0) * P.decay[g] + (-v1).max(0.0) + (-v2).max(0.0);
            let km = ((a.month.min(11) * T + t) * G + g) * KMO;
            a.months[km + 1] = m.prod_TG[(t, g)];
            a.months[km + 2] = sold;
            a.months[km + 3] = st.inv[(t, g)];
        }
        // what each need cost: the goods bought for it, at the month's prices
        for s in 0..S {
            let mut paid = 0.0;
            for g in 0..G {
                let pr = C.props[(g, s)];
                if pr > 0.0 {
                    paid += m.share[(t, g * S + s)] / pr * m.fill[(t, g)] * p[(t, g)];
                }
            }
            a.needs[(t * S + s) * 2] += m.q_hh[(t, s)] * st.pop[t] * paid;
            a.needs[(t * S + s) * 2 + 1] += m.q_l[(t, s)] * m.lord_pop[t] * paid;
        }
    }
    // the families' land: the rents of its farms and pits (the units are in
    // order of cell, so a cell's are found by halving) and of its town houses
    // (`families::take_rents`, `families::take_urban_rents`)
    if fam_on {
        for c in 0..st.N {
            let Some(o) = families::owner_of_cell(st, c) else { continue };
            let mut u = m.ex.partition_point(|&x| x < c);
            while u < m.ex.len() && m.ex[u] == c {
                a.fam[o].rents += m.rent_ex[u];
                a.towns[m.et[u] * KT + T_RENT_FAM] += m.rent_ex[u];
                fam_rents += m.rent_ex[u];
                u += 1;
            }
            let t = st.catch[c];
            if st.urban[c] && t >= 0 && st.urban_rent[t as usize] > 0.0 {
                let t = t as usize;
                let r = m.urban_pay[t] * st.cell_urban_rent[c] / st.urban_rent[t];
                a.fam[o].rents += r;
                a.towns[t * KT + T_RENT_FAM] += r;
                fam_rents += r;
            }
        }
    }
    (b.m_rents, b.m_fam_rents) = (rents, fam_rents);
    b.stock.data.copy_from_slice(&st.inv.data);
    b.seconds += t0.elapsed().as_secs_f64();
    st.acct = b;
}

/// Every works' dividend, before the families take theirs out of the list.
pub fn note_dividends(b: &mut Books, div: &[f64]) {
    if b.on {
        b.div = div.to_vec();
    }
}

/// The month's incomes, once dividends are declared and new money issued and
/// before the purses are credited: `merch` is each town's merchants' takings
/// less what they paid out, `hh_inc` everything its households are about to
/// receive.
#[allow(clippy::too_many_arguments)]
pub fn close_month(st: &mut State, fi: &[usize], ft: &[usize], fm: &[usize], merch: &[f64], hh_inc: &[f64], fdiv: &[f64], mdiv: &[f64], fund_div: f64, issue: f64, pop_total: f64) {
    if !st.acct.on {
        return;
    }
    let t0 = Instant::now();
    let mut b = std::mem::take(&mut st.acct);
    let (T, G, _, M, _) = b.dims;
    let kinds = HH_KINDS.clone().count();
    let hk = |k: usize| k - T_PAY_WORKS;
    let a = &mut b.cur;

    // dividends: a family's works pay the family, the rest their town's households
    let mut fam_div = 0.0;
    for (k, &f) in fi.iter().enumerate() {
        let d = b.div.get(k).copied().unwrap_or(0.0);
        if d == 0.0 {
            continue;
        }
        a.methods[(ft[k] * M + fm[k]) * KM + M_DIV] += d;
        if !a.works.is_empty() {
            a.works[f * KW + W_DIV] += d;
        }
        if let Some(o) = families::owner_of_firm(st, f) {
            a.fam[o].dividends += d;
            a.towns[ft[k] * KT + T_DIV_FAM] += d;
            fam_div += d;
        }
    }
    let mut err: f64 = 0.0;
    for t in 0..T {
        let popshare = st.pop[t] / pop_total;
        let row = &mut b.mk[t * kinds..(t + 1) * kinds];
        row[hk(T_DIV_WORKS)] = fdiv[t];
        row[hk(T_DIV_MERCH)] = mdiv[t];
        row[hk(T_FUND)] = fund_div * popshare;
        row[hk(T_ISSUE)] = issue * popshare;
        let mut sum = 0.0;
        for (j, &v) in row.iter().enumerate() {
            a.towns[t * KT + T_PAY_WORKS + j] += v;
            sum += v;
        }
        // the kinds must add up to what the households are about to be paid
        err = err.max((sum - hh_inc[t]).abs() / hh_inc[t].abs().max(1.0));
        a.hh_in += hh_inc[t];
        a.towns[t * KT + T_MERCH_NET] += merch[t];
        if !st.town_active[t] {
            continue;
        }
        for g in 0..G {
            let k = (t * G + g) * KG;
            let pr = st.price[(t, g)];
            a.goods[k + G_PRICE] += pr;
            a.goods[k + G_LOW] = a.goods[k + G_LOW].min(pr);
            a.goods[k + G_HIGH] = a.goods[k + G_HIGH].max(pr);
            if st.price_anchor.rows == T {
                a.goods[k + G_ANCHOR] += st.price_anchor[(t, g)];
            }
            a.months[((a.month.min(11) * T + t) * G + g) * KMO] = pr;
        }
    }
    a.fund_div += fund_div;
    a.issue += issue;

    // rents: what tenants paid against what landlords, families and the fund received
    let (m_rents, m_fam_rents) = (b.m_rents, b.m_fam_rents);
    let lords: f64 = st.lord_income.iter().sum();
    let fund_in = st.fund_cash - b.fund_open + fund_div;
    a.lords_in += lords;
    a.fund_in += fund_in;
    err = err.max((m_rents - m_fam_rents - lords - fund_in).abs() / m_rents.abs().max(1.0));
    if families::active(st) {
        let got: f64 = st.families.iter().map(|f| f.month_acc).sum();
        err = err.max((fam_div + m_fam_rents - got).abs() / got.abs().max(1.0));
    }
    a.income_err = a.income_err.max(err);
    a.month += 1;
    b.seconds += t0.elapsed().as_secs_f64();
    st.acct = b;
}

// ---------------------------------------------------------------- the year's end

/// Close the year's books (`sim::run_year`, after the yearly step): finish the
/// means, value what is owned, check the balances, and put the year away.
pub fn close_year(st: &mut State, C: &Content, P: &Params, year: i64) {
    if !st.acct.on {
        return;
    }
    let t0 = Instant::now();
    let mut b = std::mem::take(&mut st.acct);
    let dims = b.dims;
    let (T, G, NS, M, S) = dims;
    // stock that went with a district at the year's end belongs to this year
    sync_stock(&mut b, &st.inv);
    let mut a = std::mem::take(&mut b.cur);
    let n = a.month.max(1) as f64;

    // ---- the goods: means, closing stock, and the balance
    let (mut worst, mut worst_at, mut worst_units) = (0.0f64, (0, 0), 0.0f64);
    let (mut oversold, mut all_bought) = (0.0, 0.0);
    for t in 0..T {
        for g in 0..G {
            let r = &mut a.goods[(t * G + g) * KG..(t * G + g + 1) * KG];
            r[G_STOCK] = st.inv[(t, g)];
            r[G_PRICE] /= n;
            r[G_ANCHOR] /= n;
            if !r[G_LOW].is_finite() {
                // a town not yet founded has no months: its price is the one it stands at
                (r[G_PRICE], r[G_LOW], r[G_HIGH]) = (st.price[(t, g)], st.price[(t, g)], st.price[(t, g)]);
            }
            let bought = r[G_HH] + r[G_LORDS] + r[G_FAM] + r[G_INPUTS] + r[G_FUEL] + r[G_LAND] + r[G_BUILD] + r[G_PROJ];
            let left = r[G_MADE] + r[G_RECV] + r[G_MOVED] + r[G_OVERSOLD] - bought - r[G_SENT] - r[G_SPOILED] - (r[G_STOCK] - r[G_OPEN]);
            oversold += r[G_OVERSOLD] * r[G_PRICE];
            all_bought += bought * r[G_PRICE];
            let through = r[G_OPEN] + r[G_MADE] + r[G_RECV];
            if through > 1e-6 && left.abs() / through > worst {
                (worst, worst_at, worst_units) = (left.abs() / through, (t, g), left);
            }
        }
    }
    for v in a.land.chunks_mut(KL) {
        v[L_HANDS] /= n;
    }
    for v in a.methods.chunks_mut(KM) {
        v[M_WORKS] /= n;
        v[M_CAP] /= n;
        v[M_HANDS] /= n;
    }

    // ---- the towns, and the island's sums
    let mut y = IslandYear { year, rent_by_sector: a.rent_sector.clone(), to_families: a.fam.clone(), ..IslandYear::default() };
    for t in 0..T {
        let r = &mut a.towns[t * KT..(t + 1) * KT];
        for k in [T_WAGE, T_SUPPLY, T_CARTERS, T_HANDS_LAND, T_HANDS_WORKS] {
            r[k] /= n;
        }
        r[T_POP] = if st.town_active[t] { st.pop[t] } else { 0.0 };
        r[T_STOCK_VALUE] = (0..G).map(|g| st.inv[(t, g)] * st.price[(t, g)]).sum();
        r[T_CASH_HH] = st.hh_cash[t];
        r[T_CASH_LORDS] = st.lord_cash[t];
        r[T_CASH_MERCH] = st.merch_cash[t];
        y.pay_works += r[T_PAY_WORKS];
        y.pay_land += r[T_PAY_LAND];
        y.pay_projects += r[T_PAY_PROJ];
        y.carting += r[T_CART_WORKS] + r[T_CART_LAND] + r[T_CART_TRADE];
        y.managers += r[T_MANAGERS];
        y.div_households += r[T_DIV_WORKS];
        y.div_families += r[T_DIV_FAM];
        y.merchants_div += r[T_DIV_MERCH];
        y.merchants_kept += r[T_MERCH_NET] - r[T_DIV_MERCH];
        y.rent_sites += r[T_RENT_SITES];
        y.rent_town += r[T_RENT_TOWN];
        y.spend_households += r[T_SPEND_HH];
        y.spend_landlords += r[T_SPEND_LORDS];
        y.spend_families += r[T_SPEND_FAM];
        y.invest += r[T_INVEST];
        y.projects += r[T_PROJECTS];
    }
    let profit: f64 = (0..T * M).map(|k| a.methods[k * KM + M_PROFIT]).sum();
    y.works_kept = profit - y.div_households - y.div_families;
    for s in 0..NS {
        if C.sector_mine[s] {
            y.rent_pits += a.rent_sector[s];
        } else {
            y.rent_farm += a.rent_sector[s];
        }
    }
    y.need_households = (0..S).map(|s| (0..T).map(|t| a.needs[(t * S + s) * 2]).sum()).collect();
    y.need_landlords = (0..S).map(|s| (0..T).map(|t| a.needs[(t * S + s) * 2 + 1]).sum()).collect();
    y.issue = a.issue;
    y.fund_div = a.fund_div;
    y.to_households = a.hh_in;
    y.to_landlords = a.lords_in;
    y.to_fund = a.fund_in;

    // ---- the headlines
    let rents = y.rent_farm + y.rent_pits + y.rent_sites + y.rent_town;
    let merchants = y.merchants_div + y.merchants_kept;
    y.wage_bill = y.pay_works + y.pay_land + y.pay_projects + y.carting + y.managers;
    y.income = y.wage_bill + profit + merchants + rents;
    y.spending = y.spend_households + y.spend_landlords + y.spend_families + y.rent_town + y.invest + y.projects;
    let inc = if y.income.abs() > 1e-9 { y.income } else { f64::NAN };
    y.share_work = y.wage_bill / inc;
    y.share_works = profit / inc;
    y.share_merchants = merchants / inc;
    y.share_land = rents / inc;
    y.pay_share = y.wage_bill / if y.to_households.abs() > 1e-9 { y.to_households } else { f64::NAN };

    // ---- who owns what: works as the families' net worth counts them, land at
    // `years_purchase` times the rent it paid this year
    let fam_rents: f64 = a.fam.iter().map(|f| f.rents).sum();
    for f in 0..st.F_count {
        if st.f_alive[f] {
            let v = families::works_value(st, C, f);
            y.works_value += v;
            if families::owner_of_firm(st, f).is_some() {
                y.works_value_families += v;
            }
        }
    }
    y.land_value = P.years_purchase * rents;
    y.land_value_families = P.years_purchase * fam_rents;
    y.money = Money {
        households: st.hh_cash.iter().sum(),
        landlords: st.lord_cash.iter().sum(),
        merchants: st.merch_cash.iter().sum(),
        works: (0..st.f_cash.len()).map(|f| st.f_cash[f]).sum(),
        families: families::cash_total(st),
        projects: st.projects.iter().map(|pr| pr.cash).sum(),
        fund: st.fund_cash,
    };
    let bill = if y.wage_bill > 1e-9 { y.wage_bill } else { f64::NAN };
    y.works_years = y.works_value / bill;
    y.land_years = y.land_value / bill;
    y.ownership_years = y.works_years + y.land_years;
    y.balance_goods = worst;
    y.balance_goods_at = worst_at;
    y.balance_goods_units = worst_units;
    y.balance_income = a.income_err;
    y.balance_spending = (y.income - y.spending).abs() / y.spending.abs().max(1.0);
    y.oversold_share = oversold / if all_bought > 0.0 { all_bought } else { f64::NAN };

    // ---- the headline series: the year's figure in each of its twelve months
    let k0 = ((year - b.start_year).max(0) as usize) * 12;
    for (name, v) in headline(&y) {
        if let Some(&s) = st.series.index.get(name) {
            for d in st.series.data[s].iter_mut().skip(k0).take(12) {
                *d = v;
            }
        }
    }

    // ---- put the year away
    let mut rec = YearRec {
        goods: a.goods.iter().map(|&v| v as f32).collect(),
        land: a.land.iter().map(|&v| v as f32).collect(),
        methods: a.methods.iter().map(|&v| v as f32).collect(),
        towns: a.towns.iter().map(|&v| v as f32).collect(),
        needs: a.needs.iter().map(|&v| v as f32).collect(),
        ..YearRec::default()
    };
    for from in 0..T {
        for to in 0..T {
            for g in 0..G {
                let k = ((from * T + to) * G + g) * 3;
                if a.trade[k] > 0.0 {
                    rec.trade.push(Flow { from: from as u16, to: to as u16, good: g as u16, qty: a.trade[k] as f32, value: a.trade[k + 1] as f32, carting: a.trade[k + 2] as f32 });
                }
            }
        }
    }
    for t in 0..T {
        for me in 0..M {
            for g in 0..G {
                let k = ((t * M + me) * G + g) * 3;
                if a.uses[k] > 0.0 || a.uses[k + 1] > 0.0 {
                    rec.uses.push(Use { town: t as u16, method: me as u16, good: g as u16, used: a.uses[k] as f32, burnt: a.uses[k + 1] as f32, cost: a.uses[k + 2] as f32 });
                }
            }
        }
    }
    if !a.works.is_empty() {
        for f in 0..st.F_count.min(a.works.len() / KW) {
            let w = &mut a.works[f * KW..(f + 1) * KW];
            if !(st.f_alive[f] || w[W_MONTHS] > 0.0 || w[W_INVEST] != 0.0) {
                continue;
            }
            w[W_OWNER] = st.f_owner.get(f).copied().unwrap_or(-1) as f64;
            w[W_METHOD] = st.f_method[f] as f64;
            w[W_TOWN] = st.f_town[f] as f64;
            w[W_CAP] = st.f_K[f];
            w[W_HANDS] /= n;
            w[W_UTIL] /= w[W_MONTHS].max(1.0);
            w[W_CASH] = st.f_cash[f];
            rec.works_id.push(f as u32);
            rec.works.extend(w.iter().map(|&v| v as f32));
        }
        b.last_works_in = a.works_in[..st.F_count.min(a.works.len() / KW) * G * 3].iter().map(|&v| v as f32).collect();
    }
    b.last_months = a.months.iter().map(|&v| v as f32).collect();
    b.years.push(y);
    b.hist.push(rec);
    // the same tables, wiped, for the year to come
    reset(&mut a, dims, st.F_count, st.families.len());
    open_stock(&mut a, &st.inv);
    b.cur = a;
    b.seconds += t0.elapsed().as_secs_f64();
    st.acct = b;
}

/// The headline series and their values for a year.
fn headline(y: &IslandYear) -> [(&'static str, f64); 8] {
    [
        ("acct_share_work", y.share_work),
        ("acct_share_works", y.share_works),
        ("acct_share_merchants", y.share_merchants),
        ("acct_share_land", y.share_land),
        ("acct_pay_share", y.pay_share),
        ("acct_works_years", y.works_years),
        ("acct_land_years", y.land_years),
        ("acct_ownership_years", y.ownership_years),
    ]
}

/// The headline series, added after everyone else's so those keep their places
/// (`observer::init`, only when the mechanism is on). Each holds the year's
/// figure in all twelve of its months, written when the year's books close.
pub fn add_series(reg: &mut Series, n_months: usize) {
    reg.add("acct_share_work", "Work's share of income", "distribution", "share", n_months);
    reg.add("acct_share_works", "Works' owners' share of income", "distribution", "share", n_months);
    reg.add("acct_share_merchants", "Merchants' share of income", "distribution", "share", n_months);
    reg.add("acct_share_land", "Land's owners' share of income", "distribution", "share", n_months);
    reg.add("acct_pay_share", "Share of household income that is pay", "distribution", "share", n_months);
    reg.add("acct_works_years", "All works, in years of the wage bill", "distribution", "years", n_months);
    reg.add("acct_land_years", "All land, in years of the wage bill", "distribution", "years", n_months);
    reg.add("acct_ownership_years", "Works and land, in years of the wage bill", "distribution", "years", n_months);
}

/// Facts for the sweep's table, under "Work and wages": the four income shares,
/// the pay share and ownership in years of wages at the marks, and the books'
/// worst balance. None unless the run kept books.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (st, C) = (&r.state, &r.content);
    let mut out = Vec::new();
    if !st.acct.on {
        return out;
    }
    let y0 = C.start_year;
    let y1 = y0 + r.rec.n_years as i64 - 1;
    let at = |name: &str, y: i64| st.series.index.get(name).map_or(f64::NAN, |&k| annual(&st.series.data[k], y0, y));
    for (name, kind) in [
        ("share_work", Kind::Share),
        ("share_works", Kind::Share),
        ("share_merchants", Kind::Share),
        ("share_land", Kind::Share),
        ("pay_share", Kind::Share),
        ("ownership_years", Kind::Real),
    ] {
        for &y in &crate::facts::marks(y0, y1) {
            out.push((format!("income_{name}_{y}"), kind, at(&format!("acct_{name}"), y)));
        }
    }
    let worst = |f: fn(&IslandYear) -> f64| st.acct.years.iter().map(f).fold(0.0, f64::max);
    out.push(("books_goods_worst".into(), Kind::Real, worst(|y| y.balance_goods)));
    out.push(("books_income_worst".into(), Kind::Real, worst(|y| y.balance_income.max(y.balance_spending))));
    out
}

// ---------------------------------------------------------------- what the viewer asks for

/// One good in one town's books for a year. Quantities are the year's totals.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct GoodBooks {
    pub good: usize,
    pub name: String,
    pub made: f64,
    /// Of `made`, timber from woodland cleared.
    pub cleared: f64,
    pub households: f64,
    pub landlords: f64,
    pub families: f64,
    pub works_inputs: f64,
    pub works_fuel: f64,
    pub land: f64,
    pub building: f64,
    pub projects: f64,
    /// All eight buyers together.
    pub bought: f64,
    pub sent: f64,
    pub received: f64,
    pub spoiled: f64,
    pub moved: f64,
    /// Sold though it was not in store.
    pub oversold: f64,
    pub stock_open: f64,
    pub stock: f64,
    pub asked: f64,
    /// Asked for and not had.
    pub unmet: f64,
    pub price: f64,
    pub price_low: f64,
    pub price_high: f64,
    /// The merchants' anchor (None unless the prices mechanism is on).
    pub anchor: Option<f64>,
    /// made + received + moved + oversold - bought - sent - spoiled - (stock - stock_open):
    /// what the books cannot place (rounding; these tables are kept in single precision).
    pub unplaced: f64,
}

/// One land sector in one town's books for a year.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct LandBooks {
    pub sector: usize,
    pub name: String,
    /// The good it makes.
    pub good: usize,
    pub out: f64,
    pub value: f64,
    pub hands: f64,
    pub wages: f64,
    pub carting: f64,
    pub inputs: f64,
    pub rent: f64,
}

/// One good a method (or a works) used in a year.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct WorksInput {
    pub good: usize,
    pub name: String,
    /// Used as material.
    pub used: f64,
    /// Burnt as fuel.
    pub burnt: f64,
    /// What both cost delivered to the works.
    pub cost: f64,
}

/// One method in one town's books for a year.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct MethodBooks {
    pub method: usize,
    pub name: String,
    /// The good it makes.
    pub good: usize,
    pub out: f64,
    pub value: f64,
    pub works: f64,
    pub capacity: f64,
    pub hands: f64,
    pub wages: f64,
    pub inputs: f64,
    pub fuel: f64,
    pub cart_in: f64,
    pub cart_out: f64,
    pub rent: f64,
    pub managers: f64,
    pub profit: f64,
    pub dividends: f64,
    /// What its works used, good by good.
    pub uses: Vec<WorksInput>,
}

/// A flow between this town and another.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TownFlow {
    /// The other town.
    pub town: usize,
    pub good: usize,
    pub qty: f64,
    /// At the sending town's price.
    pub value: f64,
    /// Carting paid on it (to the sending town's carters, by the receiving town's merchants).
    pub carting: f64,
}

/// Spending on one need in a town.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct NeedBooks {
    pub need: usize,
    pub name: String,
    pub households: f64,
    pub landlords: f64,
}

/// One town's books for one year.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TownBooks {
    pub year: i64,
    pub town: usize,
    pub name: String,
    /// One per good, in the content's order.
    pub goods: Vec<GoodBooks>,
    /// One per land sector, in the content's order.
    pub land: Vec<LandBooks>,
    /// The methods that worked here this year.
    pub methods: Vec<MethodBooks>,
    /// The town's own figures by name (`TOWN_FIELDS`): people, the wage, hands,
    /// what its households received by kind, rents, spending, merchants, cash.
    pub figures: Vec<(String, f64)>,
    pub needs: Vec<NeedBooks>,
    /// What the town sent, and what it received, by town and good.
    pub sent: Vec<TownFlow>,
    pub received: Vec<TownFlow>,
    /// For the last complete year only: each month's price, make, sales and
    /// closing stock of every good, `[month][good]` in `MONTH_FIELDS` order.
    pub months: Option<Vec<Vec<[f64; KMO]>>>,
}

/// One town's books for `year` (None: the last complete year). None when the
/// books are off, the year is not yet complete, or there is no such town.
pub fn town_books(st: &State, C: &Content, town: usize, year: Option<i64>) -> Option<TownBooks> {
    let b = &st.acct;
    let (T, G, NS, M, S) = b.dims;
    let k = match year {
        Some(y) => usize::try_from(y - b.start_year).ok()?,
        None => b.hist.len().checked_sub(1)?,
    };
    let rec = b.hist.get(k)?;
    if town >= T || town >= st.town_name.len() {
        return None;
    }
    let f = |v: f32| v as f64;
    let goods = (0..G)
        .map(|g| {
            let r = &rec.goods[(town * G + g) * KG..(town * G + g + 1) * KG];
            let bought: f64 = [G_HH, G_LORDS, G_FAM, G_INPUTS, G_FUEL, G_LAND, G_BUILD, G_PROJ].iter().map(|&c| f(r[c])).sum();
            GoodBooks {
                good: g,
                name: C.good_name[g].clone(),
                made: f(r[G_MADE]),
                cleared: f(r[G_CLEARED]),
                households: f(r[G_HH]),
                landlords: f(r[G_LORDS]),
                families: f(r[G_FAM]),
                works_inputs: f(r[G_INPUTS]),
                works_fuel: f(r[G_FUEL]),
                land: f(r[G_LAND]),
                building: f(r[G_BUILD]),
                projects: f(r[G_PROJ]),
                bought,
                sent: f(r[G_SENT]),
                received: f(r[G_RECV]),
                spoiled: f(r[G_SPOILED]),
                moved: f(r[G_MOVED]),
                oversold: f(r[G_OVERSOLD]),
                stock_open: f(r[G_OPEN]),
                stock: f(r[G_STOCK]),
                asked: f(r[G_ASKED]),
                unmet: (f(r[G_ASKED]) - bought).max(0.0),
                price: f(r[G_PRICE]),
                price_low: f(r[G_LOW]),
                price_high: f(r[G_HIGH]),
                anchor: (r[G_ANCHOR] > 0.0).then(|| f(r[G_ANCHOR])),
                unplaced: f(r[G_MADE]) + f(r[G_RECV]) + f(r[G_MOVED]) + f(r[G_OVERSOLD]) - bought - f(r[G_SENT]) - f(r[G_SPOILED]) - (f(r[G_STOCK]) - f(r[G_OPEN])),
            }
        })
        .collect();
    let land = (0..NS)
        .map(|s| {
            let r = &rec.land[(town * NS + s) * KL..(town * NS + s + 1) * KL];
            LandBooks {
                sector: s,
                name: C.sector_name[s].clone(),
                good: C.sector_good[s],
                out: f(r[L_OUT]),
                value: f(r[L_VALUE]),
                hands: f(r[L_HANDS]),
                wages: f(r[L_WAGES]),
                carting: f(r[L_CART]),
                inputs: f(r[L_INPUTS]),
                rent: f(r[L_RENT]),
            }
        })
        .collect();
    let methods = (0..M)
        .filter_map(|m| {
            let r = &rec.methods[(town * M + m) * KM..(town * M + m + 1) * KM];
            (r[M_WORKS] > 0.0).then(|| MethodBooks {
                method: m,
                name: C.method_name[m].clone(),
                good: C.method_out[m],
                out: f(r[M_OUT]),
                value: f(r[M_VALUE]),
                works: f(r[M_WORKS]),
                capacity: f(r[M_CAP]),
                hands: f(r[M_HANDS]),
                wages: f(r[M_WAGES]),
                inputs: f(r[M_INPUTS]),
                fuel: f(r[M_FUEL]),
                cart_in: f(r[M_CART_IN]),
                cart_out: f(r[M_CART_OUT]),
                rent: f(r[M_RENT]),
                managers: f(r[M_MANAGERS]),
                profit: f(r[M_PROFIT]),
                dividends: f(r[M_DIV]),
                uses: rec
                    .uses
                    .iter()
                    .filter(|u| u.town as usize == town && u.method as usize == m)
                    .map(|u| WorksInput { good: u.good as usize, name: C.good_name[u.good as usize].clone(), used: f(u.used), burnt: f(u.burnt), cost: f(u.cost) })
                    .collect(),
            })
        })
        .collect();
    let flow = |fl: &Flow, other: u16| TownFlow { town: other as usize, good: fl.good as usize, qty: f(fl.qty), value: f(fl.value), carting: f(fl.carting) };
    let months = (k + 1 == b.hist.len() && b.last_months.len() == 12 * T * G * KMO).then(|| {
        (0..12)
            .map(|mo| (0..G).map(|g| std::array::from_fn(|c| f(b.last_months[((mo * T + town) * G + g) * KMO + c]))).collect())
            .collect()
    });
    Some(TownBooks {
        year: b.start_year + k as i64,
        town,
        name: st.town_name[town].clone(),
        goods,
        land,
        methods,
        figures: TOWN_FIELDS.iter().enumerate().map(|(c, n)| (n.to_string(), f(rec.towns[town * KT + c]))).collect(),
        needs: (0..S)
            .map(|s| NeedBooks { need: s, name: C.service_name[s].clone(), households: f(rec.needs[(town * S + s) * 2]), landlords: f(rec.needs[(town * S + s) * 2 + 1]) })
            .collect(),
        sent: rec.trade.iter().filter(|fl| fl.from as usize == town).map(|fl| flow(fl, fl.to)).collect(),
        received: rec.trade.iter().filter(|fl| fl.to as usize == town).map(|fl| flow(fl, fl.from)).collect(),
        months,
    })
}

/// One works' books for one year.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct WorksYear {
    pub year: i64,
    /// The owning family (None: the small capital of its town).
    pub owner: Option<usize>,
    pub town: usize,
    pub capacity: f64,
    pub out: f64,
    pub value: f64,
    pub revenue: f64,
    pub inputs: f64,
    pub fuel: f64,
    pub cart_in: f64,
    pub cart_out: f64,
    pub hands: f64,
    pub wages: f64,
    pub rent: f64,
    pub managers: f64,
    pub profit: f64,
    pub dividends: f64,
    pub invest: f64,
    pub util: f64,
    pub cash: f64,
    pub months: f64,
}

/// One works' books: every complete year of its life, and what it used in the last.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct WorksBooks {
    pub firm: usize,
    pub name: String,
    pub method: usize,
    pub years: Vec<WorksYear>,
    /// The last complete year's materials and fuel by good (empty if it did not work).
    pub inputs: Vec<WorksInput>,
}

/// One works' books. None when the books (or the works' part of them) are off
/// or there is no such works.
pub fn works_books(st: &State, C: &Content, firm: usize) -> Option<WorksBooks> {
    let b = &st.acct;
    if !b.on || firm >= st.F_count {
        return None;
    }
    let G = b.dims.1;
    let f = |v: f32| v as f64;
    let mut years = Vec::new();
    for (k, rec) in b.hist.iter().enumerate() {
        let Ok(j) = rec.works_id.binary_search(&(firm as u32)) else { continue };
        let r = &rec.works[j * KW..(j + 1) * KW];
        years.push(WorksYear {
            year: b.start_year + k as i64,
            owner: (r[W_OWNER] >= 0.0).then_some(r[W_OWNER] as usize),
            town: r[W_TOWN] as usize,
            capacity: f(r[W_CAP]),
            out: f(r[W_OUT]),
            value: f(r[W_VALUE]),
            revenue: f(r[W_REVENUE]),
            inputs: f(r[W_INPUTS]),
            fuel: f(r[W_FUEL]),
            cart_in: f(r[W_CART_IN]),
            cart_out: f(r[W_CART_OUT]),
            hands: f(r[W_HANDS]),
            wages: f(r[W_WAGES]),
            rent: f(r[W_RENT]),
            managers: f(r[W_MANAGERS]),
            profit: f(r[W_PROFIT]),
            dividends: f(r[W_DIV]),
            invest: f(r[W_INVEST]),
            util: f(r[W_UTIL]),
            cash: f(r[W_CASH]),
            months: f(r[W_MONTHS]),
        });
    }
    let inputs = if (firm + 1) * G * 3 <= b.last_works_in.len() {
        (0..G)
            .filter_map(|g| {
                let r = &b.last_works_in[(firm * G + g) * 3..(firm * G + g) * 3 + 3];
                (r[0] > 0.0 || r[1] > 0.0).then(|| WorksInput { good: g, name: C.good_name[g].clone(), used: f(r[0]), burnt: f(r[1]), cost: f(r[2]) })
            })
            .collect()
    } else {
        Vec::new()
    };
    Some(WorksBooks { firm, name: st.f_name[firm].clone(), method: st.f_method[firm], years, inputs })
}

/// The island's account of the last complete year (None while the books are off or empty).
pub fn island_year(st: &State) -> Option<IslandYear> {
    st.acct.years.last().cloned()
}

// ---------------------------------------------------------------- the run file

/// A table for `run.bin`: name, shape, and its numbers as single floats or as integers.
pub enum Table {
    F32(String, Vec<usize>, Vec<f32>),
    I32(String, Vec<usize>, Vec<i32>),
}

/// The books' arrays for the run file (`record::pack`): empty unless the run
/// kept books. `lite` leaves the works out.
pub fn tables(st: &State, lite: bool) -> Vec<Table> {
    let b = &st.acct;
    let mut out = Vec::new();
    if !b.on || b.hist.is_empty() {
        return out;
    }
    let (T, G, NS, M, S) = b.dims;
    let n = b.hist.len();
    let cat = |f: fn(&YearRec) -> &Vec<f32>| b.hist.iter().flat_map(|r| f(r).iter().copied()).collect::<Vec<f32>>();
    out.push(Table::F32("acct_goods".into(), vec![n, T, G, KG], cat(|r| &r.goods)));
    out.push(Table::F32("acct_land".into(), vec![n, T, NS, KL], cat(|r| &r.land)));
    out.push(Table::F32("acct_methods".into(), vec![n, T, M, KM], cat(|r| &r.methods)));
    out.push(Table::F32("acct_towns".into(), vec![n, T, KT], cat(|r| &r.towns)));
    out.push(Table::F32("acct_needs".into(), vec![n, T, S, 2], cat(|r| &r.needs)));
    // the flows of each year are rows off[year]..off[year + 1]
    let mut off = vec![0i32];
    let (mut key, mut val) = (Vec::new(), Vec::new());
    for r in &b.hist {
        for fl in &r.trade {
            key.extend([fl.from as i32, fl.to as i32, fl.good as i32]);
            val.extend([fl.qty, fl.value, fl.carting]);
        }
        off.push((key.len() / 3) as i32);
    }
    out.push(Table::I32("acct_trade_off".into(), vec![n + 1], off));
    out.push(Table::I32("acct_trade_key".into(), vec![key.len() / 3, 3], key));
    out.push(Table::F32("acct_trade_val".into(), vec![val.len() / 3, 3], val));
    let mut off = vec![0i32];
    let (mut key, mut val) = (Vec::new(), Vec::new());
    for r in &b.hist {
        for u in &r.uses {
            key.extend([u.town as i32, u.method as i32, u.good as i32]);
            val.extend([u.used, u.burnt, u.cost]);
        }
        off.push((key.len() / 3) as i32);
    }
    out.push(Table::I32("acct_use_off".into(), vec![n + 1], off));
    out.push(Table::I32("acct_use_key".into(), vec![key.len() / 3, 3], key));
    out.push(Table::F32("acct_use_val".into(), vec![val.len() / 3, 3], val));
    if !lite && b.hist.iter().any(|r| !r.works_id.is_empty()) {
        let mut off = vec![0i32];
        let (mut id, mut val) = (Vec::new(), Vec::new());
        for r in &b.hist {
            id.extend(r.works_id.iter().map(|&f| f as i32));
            val.extend_from_slice(&r.works);
            off.push(id.len() as i32);
        }
        out.push(Table::I32("acct_works_off".into(), vec![n + 1], off));
        out.push(Table::I32("acct_works_id".into(), vec![id.len()], id.clone()));
        out.push(Table::F32("acct_works".into(), vec![id.len(), KW], val));
    }
    out
}

/// The books' part of the run file's header: the columns of every table and
/// the island's account of every year. None unless the run kept books.
pub fn header(st: &State) -> Option<Value> {
    let b = &st.acct;
    if !b.on || b.hist.is_empty() {
        return None;
    }
    Some(json!({
        "start_year": b.start_year,
        "goods_fields": GOODS_FIELDS,
        "land_fields": LAND_FIELDS,
        "method_fields": METHOD_FIELDS,
        "town_fields": TOWN_FIELDS,
        "need_fields": ["households", "landlords"],
        "trade_key": ["from", "to", "good"],
        "trade_val": ["qty", "value", "carting"],
        "use_key": ["town", "method", "good"],
        "use_val": ["used", "burnt", "cost"],
        "works_fields": WORKS_FIELDS,
        "island": serde_json::to_value(&b.years).expect("the island's account serialises"),
    }))
}
