//! Gauges on the books: work's share of income and of spending, what ownership
//! costs and who owns what. Not in the Python (`docs/rules/gauges.md`).
//!
//! The books (`accounts.rs`) say who was paid what in a year. The gauges read
//! a closed year of them and reckon what the books alone cannot say: of a unit
//! of each good's price, how much comes to rest as wages, as rent and as the
//! owners' and merchants' part once every good it was made from is followed
//! back through its own makers; work's share of income net of the plant worn
//! out; the share of what the island spends that is paid out as wages; a floor
//! under work's share; the labour inside a machine; what a wage buys of made
//! goods and of raw ones; and the families' part of the works and the land.
//!
//! It is an observer of an observer. Nothing in the economy reads
//! `State::gauges`, no random number is drawn, and a run with the gauges on is
//! the same run, number for number, as one with them off (`tests/gauges.rs`).
//! It needs the books: with `accounts.enabled` off it does nothing at all.

use serde::{Deserialize, Serialize};

use crate::accounts::{self, KG, KL, KM, KT};
use crate::content::Content;
use crate::facts::Kind;
use crate::families;
use crate::observer::{Series, annual};
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::State;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GaugesParams {
    /// Read the gauges off each year's books (needs `accounts.enabled`).
    pub enabled: bool,
}

impl Default for GaugesParams {
    /// Off: what the port runs with.
    fn default() -> GaugesParams {
        GaugesParams { enabled: false }
    }
}

impl GaugesParams {
    /// The game's gauges.
    pub fn game() -> GaugesParams {
        GaugesParams { enabled: true }
    }
}

/// The share of a method's hands whose work no machine may do, by the
/// method's name; every method and land sector not named has none. Until the
/// classes of task are built (`tasks`, stage 5 of the run to 2050), this is
/// the one number of them the floor needs, and it stands here: the first
/// value for shops and trades, a tenth. The land, the carters, the navvies and
/// the managers have none. When the classes are in the content, the floor
/// reads them from there and this table goes.
pub const PRESENCE: [(&str, f64); 1] = [("services", 0.10)];

/// Years a method counts as new work after it is found.
pub const NEW_WORK_YEARS: i64 = 40;

/// A good worth less than this share of everything made in the year is taken
/// as not made, and its price is not followed back (`reckon`).
pub const MADE_AT_LEAST: f64 = 1e-5;

/// The years over which labour's part in machinery is averaged for its fact.
pub const MACHINERY_SPAN: (i64, i64) = (1775, 1900);

/// The gauges' series: the year's figure in all twelve of its months, written
/// when the year's books close (NaN until then), as the books' own are.
pub const SERIES: [(&str, &str, &str); 17] = [
    ("gauge_work_net", "Work's share of income net of wear", "share"),
    ("gauge_invest_share", "Investment's share of spending", "share"),
    ("gauge_spend_work", "Share of spending paid out as wages", "share"),
    ("gauge_basket_work", "Share of households' spending paid out as wages", "share"),
    ("gauge_spend_by_pay", "Share of spending paid for out of pay", "share"),
    ("gauge_floor", "Floor under work's share", "share"),
    ("gauge_plant_years", "Works in years of income", "years"),
    ("gauge_labour_in_machinery", "Labour inside machinery", "share"),
    ("gauge_labour_in_plant", "Labour inside new plant", "share"),
    ("gauge_fork_made", "What a wage buys of made goods (first year = 1)", "ratio"),
    ("gauge_fork_raw", "What a wage buys of raw goods (first year = 1)", "ratio"),
    ("gauge_fork_all", "What a wage buys of all goods (first year = 1)", "ratio"),
    ("gauge_hand_hours", "Hours worked over hours by hand", "share"),
    ("gauge_new_work", "Hands in methods found in forty years", "share"),
    ("gauge_lost_services", "Services made and lost", "share"),
    ("gauge_families_works", "Families' share of the works", "share"),
    ("gauge_families_land", "Families' share of the land", "share"),
];

/// One good's place in the year's cost system, island-wide. Shares are of
/// the value of what was made (at the prices of the towns it was made in) and
/// of the carting paid to take it between towns.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct GoodCost {
    pub good: usize,
    pub name: String,
    /// What it was worth, made and carted between towns, in the year.
    pub value: f64,
    /// Of a unit of its price: wages (its carters' and managers' pay among
    /// them), rent, and the rest (the owners' profit after wear, and the
    /// merchants' spread). With `inputs` they add up to one.
    pub wages: f64,
    pub rent: f64,
    pub rest: f64,
    /// Of a unit of its price, each good it was made from, in the content's
    /// order: materials, fuel, the land's machinery and rails, and the plant
    /// worn out (`wear`, which is among them).
    pub inputs: Vec<f64>,
    pub wear: f64,
    /// The same unit followed back through every good it was made from: what
    /// comes to rest as wages, as rent, and as the rest. They add up to one.
    pub chain_wages: f64,
    pub chain_rent: f64,
    pub chain_rest: f64,
    /// Spoiled in store over made, by quantity.
    pub lost: f64,
    /// Worker-months a unit in works now, over worker-months a unit by the
    /// good's hand method (the method of the first year that takes the most
    /// work). NaN for a good made on the land or with no hand method.
    pub hand_hours: f64,
}

/// One town's gauges.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TownGauge {
    pub town: usize,
    /// Pay over the income earned in the town (pay, its works' profit, its
    /// merchants' takings, its rents).
    pub work_gross: f64,
    /// The share of what was bought in the town that comes to rest as wages,
    /// through the island's chain.
    pub spend_work: f64,
}

/// The gauges of one year. Shares are of the year's totals.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct GaugeYear {
    pub year: i64,
    // ---- work's share of income
    /// The books' `share_work`: the wage bill over income.
    pub work_gross: f64,
    /// The plant worn out in the year: every works' capacity times
    /// `depreciation` times what a unit of capacity costs at the year's prices.
    pub wear: f64,
    /// The wage bill over income less wear.
    pub work_net: f64,
    /// Works' and railways' and canals' outlay over spending.
    pub invest_share: f64,
    // ---- the share of spending paid out as wages
    /// Everything bought for use (by households, landlords and families, by
    /// works being built, by railways and canals, house rent), weighted by each
    /// good's wages through its whole chain.
    pub spend_work: f64,
    pub spend_rent: f64,
    pub spend_rest: f64,
    /// The same for what households bought, house rent among it.
    pub basket_work: f64,
    /// Households' spending times the part of their income that was pay, over
    /// all spending: the part of spending paid for out of pay.
    pub spend_by_pay: f64,
    /// Work's share of spending if every hand's work but that which no
    /// machine may do (`PRESENCE`) cost nothing, each good's price falling
    /// with it, with recipes and the money shares of spending held.
    pub floor: f64,
    /// That work's part of spending at today's prices.
    pub presence_today: f64,
    // ---- plant
    /// Every works at its book value over the year's income.
    pub plant_years: f64,
    /// Of a unit of machinery's price, what comes to rest as wages and as rent.
    pub labour_in_machinery: f64,
    pub land_in_machinery: f64,
    /// The same for what works being built bought, as one bundle.
    pub labour_in_plant: f64,
    // ---- what a wage buys
    /// The people-weighted mean wage over the first year's.
    pub wage_index: f64,
    /// Chained Fisher price indexes of what households bought, first year = 1:
    /// goods made in works, goods of the land, and all of them.
    pub price_made: f64,
    pub price_raw: f64,
    pub price_all: f64,
    /// `wage_index` over each.
    pub fork_made: f64,
    pub fork_raw: f64,
    pub fork_all: f64,
    // ---- hands
    /// Worker-months in works over what the goods they made would have taken
    /// by each good's hand method, before learning.
    pub hand_hours: f64,
    /// Hands in methods found in the last `NEW_WORK_YEARS` years over all hands.
    pub new_work: f64,
    /// Spoiled over made: services, and every good by value.
    pub lost_services: f64,
    pub lost_all: f64,
    // ---- who owns what
    /// Each family's share of all works' book value, and of all land's value
    /// (the books' `years_purchase` times the rent it was paid).
    pub families_works: Vec<f64>,
    pub families_land: Vec<f64>,
    // ---- the reckoning
    pub goods: Vec<GoodCost>,
    pub towns: Vec<TownGauge>,
    /// The worst good's |chain wages + chain rent + chain rest - 1|: the
    /// solve's own check.
    pub chain_error: f64,
}

/// `State::gauges`: empty while the mechanism is off.
#[derive(Clone, Debug, Default)]
pub struct Gauges {
    /// Set by `init` when `gauges.enabled` and `accounts.enabled`.
    pub on: bool,
    /// Every complete year's gauges.
    pub years: Vec<GaugeYear>,
    /// The chain of prices: last year's island prices and households'
    /// quantities by good, the three levels, and the first year's wage.
    last: Option<(Vec<f64>, Vec<f64>)>,
    level: [f64; 3],
    wage0: f64,
}

fn presence(C: &Content, m: usize) -> f64 {
    PRESENCE.iter().find(|(n, _)| *n == C.method_name[m]).map_or(0.0, |&(_, v)| v)
}

/// The hand method of each good: of the methods known in the first year that
/// make it, the one that takes the most work a unit. (When the production
/// tree's overlay lands, its `hand` mark on a method says this instead.)
pub fn hand_methods(C: &Content) -> Vec<Option<usize>> {
    (0..C.G)
        .map(|g| {
            (0..C.M)
                .filter(|&m| C.method_out[m] == g && C.tech_known[m])
                .max_by(|&a, &b| C.m_labor[a].total_cmp(&C.m_labor[b]))
        })
        .collect()
}

/// Make ready (`sim::start_from`, after the books open). Does nothing unless
/// both the gauges and the books are on.
pub fn init(st: &mut State, P: &Params) {
    if !(P.gauges.enabled && P.accounts.enabled) {
        return;
    }
    st.gauges = Gauges { on: true, level: [1.0; 3], ..Gauges::default() };
}

/// The series, after the books' own (`observer::init`), only when the gauges are on.
pub fn add_series(reg: &mut Series, P: &Params, n_months: usize) {
    if !(P.gauges.enabled && P.accounts.enabled) {
        return;
    }
    for (name, label, unit) in SERIES {
        reg.add(name, label, "distribution", unit, n_months);
    }
}

/// Solve `(I - a) x = b` for each column of `b` (Gauss with partial pivoting).
/// `a` is `n * n`, row-major; `b` is `n * k`. None if the system is singular.
fn solve(n: usize, a: &[f64], b: &[f64], k: usize) -> Option<Vec<f64>> {
    let mut m: Vec<f64> = (0..n * n).map(|ij| if ij / n == ij % n { 1.0 } else { 0.0 } - a[ij]).collect();
    let mut x = b.to_vec();
    for c in 0..n {
        let p = (c..n).max_by(|&i, &j| m[i * n + c].abs().total_cmp(&m[j * n + c].abs()))?;
        if m[p * n + c].abs() < 1e-12 {
            return None;
        }
        if p != c {
            for j in 0..n {
                m.swap(p * n + j, c * n + j);
            }
            for j in 0..k {
                x.swap(p * k + j, c * k + j);
            }
        }
        for i in (0..n).filter(|&i| i != c) {
            let f = m[i * n + c] / m[c * n + c];
            if f != 0.0 {
                for j in c..n {
                    m[i * n + j] -= f * m[c * n + j];
                }
                for j in 0..k {
                    x[i * k + j] -= f * x[c * k + j];
                }
            }
        }
    }
    for i in 0..n {
        for j in 0..k {
            x[i * k + j] /= m[i * n + i];
        }
    }
    Some(x)
}

fn ratio(a: f64, b: f64) -> f64 {
    if b.abs() > 1e-12 { a / b } else { f64::NAN }
}

/// Read the gauges off the year the books have just closed (`sim::end_year`,
/// after `accounts::close_year`).
pub fn close_year(st: &mut State, C: &Content, P: &Params, year: i64) {
    if !st.gauges.on || !st.acct.on || st.acct.hist.n == 0 {
        return;
    }
    let mut ga = std::mem::take(&mut st.gauges);
    let y = reckon(st, C, P, year, &mut ga);
    let k0 = ((year - st.acct.start_year).max(0) as usize) * 12;
    for (name, v) in headline(&y) {
        if let Some(&s) = st.series.index.get(name) {
            for d in st.series.data[s].iter_mut().skip(k0).take(12) {
                *d = v;
            }
        }
    }
    ga.years.push(y);
    st.gauges = ga;
}

fn headline(y: &GaugeYear) -> [(&'static str, f64); 17] {
    let fam = |v: &[f64]| v.iter().sum::<f64>();
    [
        ("gauge_work_net", y.work_net),
        ("gauge_invest_share", y.invest_share),
        ("gauge_spend_work", y.spend_work),
        ("gauge_basket_work", y.basket_work),
        ("gauge_spend_by_pay", y.spend_by_pay),
        ("gauge_floor", y.floor),
        ("gauge_plant_years", y.plant_years),
        ("gauge_labour_in_machinery", y.labour_in_machinery),
        ("gauge_labour_in_plant", y.labour_in_plant),
        ("gauge_fork_made", y.fork_made),
        ("gauge_fork_raw", y.fork_raw),
        ("gauge_fork_all", y.fork_all),
        ("gauge_hand_hours", y.hand_hours),
        ("gauge_new_work", y.new_work),
        ("gauge_lost_services", y.lost_services),
        ("gauge_families_works", fam(&y.families_works)),
        ("gauge_families_land", fam(&y.families_land)),
    ]
}

/// The gauges' series' figures for `year`, each with its place in the series
/// registry, as `accounts::year_series` gives the books' (`live::live_year`).
pub fn year_series(st: &State, year: i64) -> Vec<(usize, f64)> {
    let Some(y) = st.gauges.years.iter().rev().find(|g| g.year == year) else { return Vec::new() };
    headline(y).iter().filter_map(|&(name, v)| st.series.index.get(name).map(|&s| (s, v))).collect()
}

/// The gauges of the last complete year (None while they are off or empty).
pub fn gauge_year(st: &State) -> Option<GaugeYear> {
    st.gauges.years.last().cloned()
}

/// Everything for one year, from the books' last complete year.
fn reckon(st: &State, C: &Content, P: &Params, year: i64, ga: &mut Gauges) -> GaugeYear {
    let b = &st.acct;
    let rec = b.year(b.hist.n - 1).expect("the year the books have just closed");
    let isl = b.years.last().expect("the island's account of that year");
    let (T, G, NS, M, _) = (st.T_max, C.G, C.NS, C.M, C.S);
    let f = |v: f32| v as f64;
    let good = |t: usize, g: usize, c: usize| f(rec.goods[(t * G + g) * KG + c]);
    let gcol = |name: &str| accounts::GOODS_FIELDS.iter().position(|&n| n == name).expect("a goods column");
    let tcol = |name: &str| accounts::TOWN_FIELDS.iter().position(|&n| n == name).expect("a town column");
    let mcol = |name: &str| accounts::METHOD_FIELDS.iter().position(|&n| n == name).expect("a method column");
    let lcol = |name: &str| accounts::LAND_FIELDS.iter().position(|&n| n == name).expect("a land column");
    let (g_price, g_made, g_spoiled, g_land) = (gcol("price"), gcol("made"), gcol("spoiled"), gcol("land"));
    let buyers = [gcol("households"), gcol("landlords"), gcol("families"), gcol("building"), gcol("projects")];
    let (g_hh, g_build) = (gcol("households"), gcol("building"));
    let towns: Vec<usize> = (0..T.min(st.town_active.len())).filter(|&t| st.town_active[t]).collect();
    let price = |t: usize, g: usize| good(t, g, g_price);
    let tf = |t: usize, c: usize| f(rec.towns[t * KT + c]);

    // ---- the cost system: for each good, its value and what went into it
    let mut value = vec![0.0; G];
    let mut wages = vec![0.0; G];
    let mut presence_wages = vec![0.0; G];
    let mut rent = vec![0.0; G];
    let mut rest = vec![0.0; G];
    let mut inputs = vec![0.0; G * G]; // [made good][input good], money
    let mut wear_g = vec![0.0; G];
    let mut wear_total = 0.0;
    let (gm, gr) = (C.gi("machinery"), C.gi("rails"));
    let spread = P.merchant_spread;
    for &t in &towns {
        // the land: value = spread + wages + carting + machinery and rails + rent
        let (mq, rq) = (good(t, gm, g_land) * price(t, gm), good(t, gr, g_land) * price(t, gr));
        let mshare = ratio(mq, mq + rq);
        for s in 0..NS {
            let r = &rec.land[(t * NS + s) * KL..(t * NS + s + 1) * KL];
            let g = C.sector_good[s];
            let v = f(r[lcol("value")]);
            value[g] += v;
            wages[g] += f(r[lcol("wages")]) + f(r[lcol("carting")]);
            rent[g] += f(r[lcol("rent")]);
            rest[g] += v * spread;
            let inp = f(r[lcol("inputs")]);
            if inp != 0.0 {
                let ms = if mshare.is_finite() { mshare } else { 1.0 };
                inputs[g * G + gm] += inp * ms;
                inputs[g * G + gr] += inp * (1.0 - ms);
            }
        }
        // the works: value = spread + carting out + materials and fuel + carting in
        // + wages + rent + managers + profit, and of the profit the plant worn out
        for m in 0..M {
            let r = &rec.methods[(t * M + m) * KM..(t * M + m + 1) * KM];
            if r[mcol("works")] <= 0.0 && r[mcol("value")] == 0.0 {
                continue;
            }
            let g = C.method_out[m];
            let v = f(r[mcol("value")]);
            value[g] += v;
            let w = f(r[mcol("wages")]);
            wages[g] += w + f(r[mcol("cart_in")]) + f(r[mcol("cart_out")]) + f(r[mcol("managers")]);
            presence_wages[g] += w * presence(C, m);
            rent[g] += f(r[mcol("rent")]);
            // materials and fuel by good, in proportion to what each cost delivered
            let paid = f(r[mcol("inputs")]) + f(r[mcol("fuel")]);
            let uses: Vec<&accounts::Use> = rec.uses.iter().filter(|u| u.town as usize == t && u.method as usize == m).collect();
            let delivered: f64 = uses.iter().map(|u| f(u.cost)).sum();
            for u in &uses {
                inputs[g * G + u.good as usize] += paid * ratio(f(u.cost), delivered).max(0.0);
            }
            // the plant worn out, in its goods at the town's prices
            let cap = f(r[mcol("capacity")]);
            let mut wear = 0.0;
            for h in 0..G {
                let x = cap * P.depreciation * C.m_cap[(m, h)] * price(t, h);
                if x != 0.0 {
                    inputs[g * G + h] += x;
                    wear += x;
                }
            }
            wear_g[g] += wear;
            wear_total += wear;
            rest[g] += v * spread + f(r[mcol("profit")]) - wear;
        }
    }
    // carting between towns: the receiving town's merchants pay the sending town's carters
    for fl in rec.trade {
        let g = fl.good as usize;
        value[g] += fl.carting as f64;
        wages[g] += fl.carting as f64;
    }

    // ---- the whole chain: (I - a) x = direct shares, for wages, rent, the rest
    // and the floor's wages
    // A good made in next to no quantity (charcoal in its last years, when a few
    // works stand at work and make almost nothing) would carry the whole of its
    // works' wear on a value of almost nothing, and one such row is enough to
    // make the solve meaningless. Under a hundred-thousandth of all that was
    // made, a good is taken as not made.
    let made_total: f64 = value.iter().sum();
    let is_made: Vec<bool> = (0..G).map(|g| value[g] > MADE_AT_LEAST * made_total).collect();
    let mut a = vec![0.0; G * G];
    let mut rhs = vec![0.0; G * 4];
    for g in 0..G {
        if is_made[g] {
            for h in 0..G {
                a[g * G + h] = inputs[g * G + h] / value[g];
            }
            rhs[g * 4] = wages[g] / value[g];
            rhs[g * 4 + 1] = rent[g] / value[g];
            rhs[g * 4 + 2] = rest[g] / value[g];
            rhs[g * 4 + 3] = presence_wages[g] / value[g];
        } else {
            // not made this year: nothing to follow back, all of it "the rest"
            rhs[g * 4 + 2] = 1.0;
        }
    }
    let x = solve(G, &a, &rhs, 4).unwrap_or_else(|| vec![f64::NAN; G * 4]);
    let lam = |g: usize| x[g * 4];
    let rho = |g: usize| x[g * 4 + 1];
    let omg = |g: usize| x[g * 4 + 2];
    // the floor: each good's new price (per unit of today's) and its wages in it
    let floor_price = |g: usize| x[g * 4 + 3] + x[g * 4 + 1] + x[g * 4 + 2];
    let chain_error = (0..G).map(|g| (lam(g) + rho(g) + omg(g) - 1.0).abs()).fold(0.0, f64::max);

    // ---- spending, through the chain
    let mut final_g = vec![0.0; G];
    let mut hh_g = vec![0.0; G];
    let mut build_g = vec![0.0; G];
    let (mut navvies, mut house_rent) = (0.0, 0.0);
    let (c_proj, c_rent_town) = (tcol("pay_projects"), tcol("rent_town"));
    let mut town_gauges = Vec::with_capacity(towns.len());
    // a town's income: what its households were paid for work, its works'
    // profit, its merchants' takings and the rents paid in it
    let pay_cols: Vec<usize> = ["pay_works", "pay_land", "pay_projects", "cart_works", "cart_land", "cart_trade", "managers"].iter().map(|n| tcol(n)).collect();
    let other_cols: Vec<usize> = ["merchants_net", "rent_land", "rent_sites", "rent_town"].iter().map(|n| tcol(n)).collect();
    for &t in &towns {
        let (mut spent, mut work) = (0.0, 0.0);
        for g in 0..G {
            let p = price(t, g);
            let q: f64 = buyers.iter().map(|&c| good(t, g, c)).sum();
            final_g[g] += q * p;
            hh_g[g] += good(t, g, g_hh) * p;
            build_g[g] += good(t, g, g_build) * p;
            spent += q * p;
            work += q * p * lam(g);
        }
        let (nv, hr) = (tf(t, c_proj), tf(t, c_rent_town));
        navvies += nv;
        house_rent += hr;
        let pay: f64 = pay_cols.iter().map(|&c| tf(t, c)).sum();
        let profit: f64 = (0..M).map(|m| f(rec.methods[(t * M + m) * KM + mcol("profit")])).sum();
        let others: f64 = other_cols.iter().map(|&c| tf(t, c)).sum::<f64>() + profit;
        town_gauges.push(TownGauge { town: t, work_gross: ratio(pay, pay + others), spend_work: ratio(work + nv, spent + nv + hr) });
    }
    let total_final: f64 = final_g.iter().sum::<f64>() + navvies + house_rent;
    let dot = |v: &[f64], w: &dyn Fn(usize) -> f64| (0..G).map(|g| v[g] * w(g)).sum::<f64>();
    let spend_work = ratio(dot(&final_g, &lam) + navvies, total_final);
    let spend_rent = ratio(dot(&final_g, &rho) + house_rent, total_final);
    let spend_rest = ratio(dot(&final_g, &omg), total_final);
    let basket_work = ratio(dot(&hh_g, &lam), hh_g.iter().sum::<f64>() + house_rent);
    // the floor: with the money shares of spending held, each good's spending
    // buys more of it as its price falls, and its wages in it are its floor
    // wages over its new price; the navvies' work and house rent hold no such work
    let floor = ratio(
        (0..G).map(|g| { let pf = floor_price(g); if pf > 1e-9 { final_g[g] * x[g * 4 + 3] / pf } else { 0.0 } }).sum(),
        total_final,
    );
    let presence_today = ratio((0..G).map(|g| final_g[g] * x[g * 4 + 3]).sum(), total_final);

    // ---- plant
    let works_value = isl.works_value;
    let net_income = isl.income - wear_total;
    let build_total: f64 = build_g.iter().sum();

    // ---- what a wage buys: chained Fisher indexes on what households bought
    let raw: Vec<bool> = (0..G).map(|g| C.sector_good.contains(&g)).collect();
    let mut qty = vec![0.0; G];
    let mut pr = vec![f64::NAN; G];
    for g in 0..G {
        let (q, v): (f64, f64) = towns.iter().map(|&t| (good(t, g, g_hh), good(t, g, g_hh) * price(t, g))).fold((0.0, 0.0), |(a, b), (c, d)| (a + c, b + d));
        qty[g] = q;
        if q > 0.0 {
            pr[g] = v / q;
        }
    }
    let pop: f64 = towns.iter().map(|&t| tf(t, tcol("pop"))).sum();
    let wage = ratio(towns.iter().map(|&t| tf(t, tcol("wage")) * tf(t, tcol("pop"))).sum(), pop);
    if let Some((p0, q0)) = &ga.last {
        for k in 0..3 {
            // 0: goods made in works, 1: goods of the land, 2: all of them; each good
            // bought in both years
            let kind = |g: usize| match k {
                0 => !raw[g],
                1 => raw[g],
                _ => true,
            };
            let pick = |g: usize| kind(g) && pr[g].is_finite() && p0[g].is_finite() && qty[g] > 0.0 && q0[g] > 0.0;
            let (mut l, mut p) = ([0.0; 2], [0.0; 2]);
            for g in (0..G).filter(|&g| pick(g)) {
                l[0] += pr[g] * q0[g];
                l[1] += p0[g] * q0[g];
                p[0] += pr[g] * qty[g];
                p[1] += p0[g] * qty[g];
            }
            let link = (ratio(l[0], l[1]) * ratio(p[0], p[1])).sqrt();
            if link.is_finite() && link > 0.0 {
                ga.level[k] *= link;
            }
        }
    } else {
        ga.wage0 = wage;
    }
    ga.last = Some((pr.clone(), qty.clone()));
    let wage_index = ratio(wage, ga.wage0);

    // ---- hands: by hand, and in new methods
    let hand = hand_methods(C);
    let mut goods_out = Vec::with_capacity(G);
    let (mut worked, mut by_hand) = (0.0, 0.0);
    let mut hands_new = 0.0;
    let (mut spoiled_v, mut made_v) = (0.0, 0.0);
    for g in 0..G {
        let (mut wm, mut out) = (0.0, 0.0);
        for &t in &towns {
            for m in (0..M).filter(|&m| C.method_out[m] == g) {
                let r = &rec.methods[(t * M + m) * KM..(t * M + m + 1) * KM];
                wm += 12.0 * f(r[mcol("hands")]);
                out += f(r[mcol("out")]);
                let found = st.tech_year.get(m).copied().unwrap_or(-1);
                if !C.tech_known[m] && found >= 0 && found <= year && year - found < NEW_WORK_YEARS {
                    hands_new += f(r[mcol("hands")]);
                }
            }
        }
        let hh = hand[g].map_or(f64::NAN, |m| ratio(wm, out * C.m_labor[m]));
        if hh.is_finite() && out > 0.0 {
            worked += wm;
            by_hand += out * C.m_labor[hand[g].expect("a hand method")];
        }
        let (made, spoiled): (f64, f64) = towns.iter().map(|&t| (good(t, g, g_made), good(t, g, g_spoiled))).fold((0.0, 0.0), |(a, b), (c, d)| (a + c, b + d));
        let pmean = ratio(towns.iter().map(|&t| good(t, g, g_made) * price(t, g)).sum(), made);
        if pmean.is_finite() {
            spoiled_v += spoiled * pmean;
            made_v += made * pmean;
        }
        // (a good taken as not made has no direct shares)
        let vm = if is_made[g] { value[g] } else { f64::NAN };
        goods_out.push(GoodCost {
            good: g,
            name: C.good_name[g].clone(),
            value: value[g],
            wages: ratio(wages[g], vm),
            rent: ratio(rent[g], vm),
            rest: ratio(rest[g], vm),
            inputs: (0..G).map(|h| ratio(inputs[g * G + h], vm)).collect(),
            wear: ratio(wear_g[g], vm),
            chain_wages: lam(g),
            chain_rent: rho(g),
            chain_rest: omg(g),
            lost: ratio(spoiled, made),
            hand_hours: hh,
        });
    }
    let all_hands: f64 = towns.iter().map(|&t| ["hands_land", "hands_works", "carters", "navvies"].iter().map(|n| tf(t, tcol(n))).sum::<f64>()).sum();
    let services = C.good_name.iter().position(|n| n == "services");

    // ---- who owns what
    let nf = st.families.len();
    let mut fam_works = vec![0.0; nf];
    for fi in 0..st.F_count {
        if st.f_alive[fi] {
            if let Some(o) = families::owner_of_firm(st, fi) {
                fam_works[o] += families::works_value(st, C, fi);
            }
        }
    }
    let families_works = fam_works.iter().map(|&v| ratio(v, works_value)).collect();
    let families_land = isl.to_families.iter().map(|fam| ratio(P.years_purchase * fam.rents, isl.land_value)).collect();

    GaugeYear {
        year,
        work_gross: isl.share_work,
        wear: wear_total,
        work_net: ratio(isl.wage_bill, net_income),
        invest_share: ratio(isl.invest + isl.projects, isl.spending),
        spend_work,
        spend_rent,
        spend_rest,
        basket_work,
        spend_by_pay: ratio((isl.spend_households + isl.rent_town) * isl.pay_share, isl.spending),
        floor,
        presence_today,
        plant_years: ratio(works_value, isl.income),
        labour_in_machinery: lam(gm),
        land_in_machinery: rho(gm),
        labour_in_plant: ratio(dot(&build_g, &lam), build_total),
        wage_index,
        price_made: ga.level[0],
        price_raw: ga.level[1],
        price_all: ga.level[2],
        fork_made: wage_index / ga.level[0],
        fork_raw: wage_index / ga.level[1],
        fork_all: wage_index / ga.level[2],
        hand_hours: ratio(worked, by_hand),
        new_work: ratio(hands_new, all_hands),
        lost_services: services.map_or(f64::NAN, |g| goods_out[g].lost),
        lost_all: ratio(spoiled_v, made_v),
        families_works,
        families_land,
        goods: goods_out,
        towns: town_gauges,
        chain_error,
    }
}

/// Facts for the sweep's table, under "Work and wages": each gauge at the
/// marks, labour's part in machinery over `MACHINERY_SPAN`, and the solve's
/// worst check. None unless the run kept the gauges.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (st, C) = (&r.state, &r.content);
    let mut out = Vec::new();
    if !st.gauges.on {
        return out;
    }
    let y0 = C.start_year;
    let y1 = y0 + r.rec.n_years as i64 - 1;
    let at = |name: &str, y: i64| st.series.index.get(name).map_or(f64::NAN, |&k| annual(&st.series.data[k], y0, y));
    for (name, _, unit) in SERIES {
        let kind = match unit {
            "share" => Kind::Share,
            "ratio" => Kind::Ratio,
            _ => Kind::Real,
        };
        for &y in &crate::facts::marks(y0, y1) {
            out.push((format!("{name}_{y}"), kind, at(name, y)));
        }
    }
    let (a, z) = MACHINERY_SPAN;
    let span: Vec<f64> = st.gauges.years.iter().filter(|g| g.year >= a && g.year <= z && g.labour_in_machinery.is_finite()).map(|g| g.labour_in_machinery).collect();
    out.push((format!("gauge_labour_in_machinery_{a}_{z}"), Kind::Share, if span.is_empty() { f64::NAN } else { span.iter().sum::<f64>() / span.len() as f64 }));
    out.push(("gauges_chain_worst".into(), Kind::Real, st.gauges.years.iter().map(|g| g.chain_error).fold(0.0, f64::max)));
    out
}

/// The gauges' part of the run file's header: every year's. None unless the
/// run kept them.
pub fn header(st: &State) -> Option<serde_json::Value> {
    if !st.gauges.on || st.gauges.years.is_empty() {
        return None;
    }
    Some(serde_json::json!({
        "presence": PRESENCE,
        "years": serde_json::to_value(&st.gauges.years).expect("the gauges serialise"),
    }))
}
