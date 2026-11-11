//! The stylised facts a finished run is reduced to (PLAN.md 1.3). Not in the Python.
//!
//! One run is 150 years of a hundred monthly series, a map a year and a few
//! thousand works. To compare many runs it helps to boil each down to a short
//! list of numbers a historian would ask for: how many people, how many of them
//! in towns, when coke overtook charcoal, how long the railway kept growing, how
//! big the works got. `stylised_facts` does that for one finished run; the sweep
//! (`sweep.rs`) lines the lists up across seeds and settings.
//!
//! Nothing here is read by the simulation, and nothing is added to its loop. The
//! facts come from what a run already keeps: the observer's series, the yearly
//! snapshots (`record.rs`) and the final state. Two things nobody keeps, how
//! many people lived in towns growing at the cap and how much land lay under
//! each use, are noted between years by `gather_year`, which the sweep calls
//! from its own loop.
//!
//! Everything is worked out from the content, so another content file works: a
//! method or good named below that the content does not have is skipped.
//!
//! Every fact taken from a series in a year is the mean of that year's twelve
//! months, the same number as in the sweep's `annual.csv`. A fact with no value
//! (a railway that was never built has no first year) is `None`: an empty cell in
//! `facts.csv`, a null in `facts.json`.

use std::collections::BTreeMap;

use crate::land::{USE_DERELICT, USE_INDUSTRY, USE_URBAN};
use crate::observer::annual;
use crate::sim::{RunResult, Sim};

/// What sort of number a fact is, which decides how the table prints it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A calendar year.
    Year,
    /// A count of people, towns, works, cells or kilometres.
    Count,
    /// A share between 0 and 1.
    Share,
    /// A level relative to another year or to a method's usual size.
    Ratio,
    /// Any other number.
    Real,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Year => "year",
            Kind::Count => "count",
            Kind::Share => "share",
            Kind::Ratio => "ratio",
            Kind::Real => "real",
        }
    }

    pub fn parse(s: &str) -> Option<Kind> {
        [Kind::Year, Kind::Count, Kind::Share, Kind::Ratio, Kind::Real].into_iter().find(|k| k.as_str() == s)
    }
}

/// One number about one run.
#[derive(Clone, Debug, PartialEq)]
pub struct Fact {
    pub name: String,
    /// The heading it is printed under.
    pub group: &'static str,
    pub kind: Kind,
    pub value: Option<f64>,
}

/// The headings, in the order they are printed.
pub const GROUPS: [&str; 9] = [
    "People and towns",
    "Work and wages",
    "Railways",
    "Works",
    "Land",
    "Methods and inventions",
    "Prices and cycles",
    "News",
    "Speed",
];

/// The two headings the sweep needs by name: it adds the news facts itself, and
/// leaves the speed out when it counts what changed between two sweeps.
pub const NEWS: &str = GROUPS[7];
pub const SPEED: &str = GROUPS[8];

/// PLAN.md 1.3: a method's share of all its good's output in a given year. Coke's
/// share of iron in 1800, powered mills' of cloth in 1830, the converter's of steel
/// in 1880, rolled rails' of rails in 1850 and steel rails' in 1890.
pub const METHOD_SHARES: [(&str, i64); 5] =
    [("coke_furnace", 1800), ("textile_mill", 1830), ("bessemer", 1880), ("rolling_mill", 1850), ("steel_rolling", 1890)];

/// Goods whose price swings are measured.
pub const PRICE_GOODS: [&str; 4] = ["grain", "iron", "rails", "cloth"];

/// The first ten years are left out of the price swings: prices are still settling.
pub const PRICE_FROM: i64 = 1760;

/// Besides the fifty-year marks, the railway's length is taken in this year
/// (PLAN.md 1.3: rail km by 1870).
pub const RAIL_YEAR: i64 = 1870;

/// The facts of one run.
#[derive(Clone, Debug, Default)]
pub struct Facts {
    pub facts: Vec<Fact>,
    /// Methods not known at the start that never produced anything.
    pub never_built: Vec<String>,
    /// Number of news items of each kind.
    pub news: BTreeMap<String, usize>,
}

impl Facts {
    pub fn get(&self, name: &str) -> Option<f64> {
        self.facts.iter().find(|f| f.name == name).and_then(|f| f.value)
    }
}

/// What the sweep's loop notes after each year, because nothing else keeps it.
#[derive(Clone, Debug, Default)]
pub struct Gathered {
    /// Per year: the share of people living in towns whose growth was held at `growth_cap`.
    pub capped: Vec<f64>,
    /// Per year: the land under each sector and, last, the land left wild, in cells
    /// (every cell's share summed). The yearly snapshot keeps only each cell's main use.
    pub area: Vec<Vec<f64>>,
}

/// Call after each `sim::run_year`.
pub fn gather_year(g: &mut Gathered, sim: &Sim) {
    let (st, P) = (&sim.st, &sim.params);
    let (mut capped, mut all) = (0.0, 0.0);
    for t in 0..st.T_max {
        if !st.town_active[t] || st.y_ratio[t] <= 0.0 {
            continue;
        }
        // the rule in people::growth before its clamp (a famine's two points off are not seen here)
        let rate = P.growth_slope * st.y_ratio[t].max(1e-3).ln() + P.growth_base;
        all += st.pop[t];
        if rate >= P.growth_cap {
            capped += st.pop[t];
        }
    }
    g.capped.push(if all > 0.0 { capped / all } else { f64::NAN });
    g.area.push((0..st.lam.rows).map(|r| (0..st.N).map(|c| st.lam[(r, c)]).sum()).collect());
}

/// The years facts are taken in: the first, every fiftieth year after it, and the
/// last. 1750, 1800, 1850 and 1900 for a whole run.
pub fn marks(y0: i64, y1: i64) -> Vec<i64> {
    let mut v = vec![y0];
    let mut y = (y0.div_euclid(50) + 1) * 50;
    while y < y1 {
        v.push(y);
        y += 50;
    }
    if y1 > y0 {
        v.push(y1);
    }
    v
}

/// Every observer series averaged over each year's twelve months: `[series][year]`,
/// in the registry's order. NaN where a share had an empty total all year.
pub fn annual_means(r: &RunResult) -> Vec<Vec<f64>> {
    let y0 = r.content.start_year;
    r.state.series.data.iter().map(|d| (0..r.rec.n_years).map(|k| annual(d, y0, y0 + k as i64)).collect()).collect()
}

/// numpy's default quantile of a sorted slice (linear between neighbours).
fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let pos = q * (sorted.len() - 1) as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f64)
}

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 }
}

/// Series looked up by name; empty when the registry has no such series.
struct Named<'a> {
    index: &'a std::collections::HashMap<String, usize>,
    data: &'a [Vec<f64>],
}

impl<'a> Named<'a> {
    fn get(&self, name: &str) -> &'a [f64] {
        self.index.get(name).map_or(&[][..], |&k| &self.data[k])
    }
}

/// Collects facts under the current heading.
struct Builder {
    facts: Vec<Fact>,
    group: &'static str,
}

impl Builder {
    fn num(&mut self, name: String, kind: Kind, v: f64) {
        self.facts.push(Fact { name, group: self.group, kind, value: if v.is_finite() { Some(v) } else { None } });
    }

    fn year(&mut self, name: String, y: Option<i64>) {
        self.facts.push(Fact { name, group: self.group, kind: Kind::Year, value: y.map(|y| y as f64) });
    }
}

/// The stylised facts of a finished run. `seconds` is how long it took.
pub fn stylised_facts(r: &RunResult, g: &Gathered, seconds: f64) -> Facts {
    let (st, C, rec) = (&r.state, &r.content, &r.rec);
    let y0 = C.start_year;
    let n = rec.n_years;
    let y1 = y0 + n as i64 - 1;
    let years: Vec<i64> = (y0..=y1).collect();
    let marks = marks(y0, y1);
    let ann = annual_means(r);
    let yearly = Named { index: &st.series.index, data: &ann };
    let series = |name: &str| yearly.get(name);
    let at = |name: &str, y: i64| -> f64 { yearly.get(name).get((y - y0) as usize).copied().unwrap_or(f64::NAN) };
    let mut b = Builder { facts: Vec::new(), group: GROUPS[0] };

    // ---- people and towns
    for &y in &marks {
        b.num(format!("pop_{y}"), Kind::Count, at("pop", y));
    }
    let capped: Vec<f64> = g.capped.iter().copied().filter(|v| v.is_finite()).collect();
    // a year sat at the cap if most people lived in towns growing at it
    b.num("growth_capped_years".into(), Kind::Share, mean(&capped.iter().map(|&v| if v >= 0.5 { 1.0 } else { 0.0 }).collect::<Vec<_>>()));
    b.num("growth_capped_people".into(), Kind::Share, mean(&capped));
    for &y in &marks {
        b.num(format!("urban_share_{y}"), Kind::Share, at("urban_share", y));
    }
    for &y in &marks {
        b.num(format!("towns_{y}"), Kind::Count, at("towns", y));
    }
    let last_town = (0..st.T_max).filter(|&t| st.town_active[t] && st.town_founded[t] >= y0).map(|t| st.town_founded[t]).max();
    b.year("last_town_founded".into(), last_town);

    // ---- work and wages
    b.group = GROUPS[1];
    for &y in &marks {
        b.num(format!("farm_share_{y}"), Kind::Share, at("ag_share", y));
    }
    for &y in &marks {
        b.num(format!("industry_share_{y}"), Kind::Share, at("ind_share", y));
    }
    for (name, label) in [("real_wage", "real_wage"), ("gdp_pc", "output_per_head")] {
        for &y in &marks[1..] {
            b.num(format!("{label}_{y}_vs_{y0}"), Kind::Ratio, at(name, y) / at(name, y0));
        }
    }
    let pc = series("gdp_pc");
    let peak = (0..pc.len()).filter(|&k| pc[k].is_finite()).max_by(|&i, &j| pc[i].total_cmp(&pc[j]));
    b.year("output_per_head_peak_year".into(), peak.map(|k| years[k]));
    // engel: where the hands and the bread go. Quarries, pits and mines are the
    // work that is neither on the land nor in the works; the rest are facts only
    // of a run that has the mechanism's series (docs/rules/engel.md).
    for &y in &marks {
        b.num(format!("mine_share_{y}"), Kind::Share, 1.0 - at("ag_share", y) - at("ind_share", y));
    }
    if C.good_index.contains_key("grain") {
        for &y in &marks[1..] {
            b.num(format!("grain_output_{y}_vs_{y0}"), Kind::Ratio, at("out_grain", y) / at("out_grain", y0));
        }
    }
    if st.series.index.contains_key("real_wage_chained") {
        for &y in &marks[1..] {
            b.num(format!("real_wage_chained_{y}_vs_{y0}"), Kind::Ratio, at("real_wage_chained", y) / at("real_wage_chained", y0));
        }
        for &y in &marks {
            b.num(format!("real_income_{y}"), Kind::Ratio, at("real_income", y));
        }
        for name in &C.service_name {
            for &y in &marks {
                b.num(format!("budget_{name}_{y}"), Kind::Share, at(&format!("budget_{name}"), y));
            }
        }
    }

    // ---- railways
    b.group = GROUPS[2];
    let km = Named { index: &st.series.index, data: &st.series.data }.get("rail_km");
    b.year("rail_first_year".into(), km.iter().position(|&v| v > 0.0).map(|t| y0 + (t / 12) as i64));
    // 1800 is left out: under the port no seed has a line before 1815, and a row of noughts says nothing
    let mut rail_years: Vec<i64> = marks[1..].iter().copied().filter(|&y| y >= 1850 || y == y1).collect();
    if y0 < RAIL_YEAR && RAIL_YEAR < y1 {
        rail_years.push(RAIL_YEAR);
    }
    rail_years.sort();
    for &y in &rail_years {
        b.num(format!("rail_km_{y}"), Kind::Count, at("rail_km", y));
    }
    // the last year that ended with more line than the year before it
    let december = |k: usize| km.get(k * 12 + 11).copied().unwrap_or(0.0);
    let grew = (0..n).rev().find(|&k| december(k) > if k == 0 { 0.0 } else { december(k - 1) });
    b.year("rail_last_growth_year".into(), grew.map(|k| years[k]));

    // ---- works
    b.group = GROUPS[3];
    let services = C.method_name.iter().position(|m| m == "services");
    for &y in &marks {
        b.num(format!("works_{y}"), Kind::Count, rec.firm_K[(y - y0) as usize].0.len() as f64);
    }
    for &y in &marks {
        let ids = &rec.firm_K[(y - y0) as usize].0;
        b.num(format!("works_ex_services_{y}"), Kind::Count, ids.iter().filter(|&&f| Some(st.f_method[f]) != services).count() as f64);
    }
    let alive: Vec<usize> = (0..st.F_count).filter(|&f| st.f_alive[f]).collect();
    b.num(format!("works_per_1000_people_{y1}"), Kind::Real, 1000.0 * alive.len() as f64 / at("pop", y1));
    // size: built capacity over the method's usual scale; works with nothing built yet are left out
    let mut size: Vec<f64> = alive.iter().filter(|&&f| st.f_K[f] > 0.0).map(|&f| st.f_K[f] / C.m_K0[st.f_method[f]]).collect();
    size.sort_by(f64::total_cmp);
    b.num(format!("works_size_median_{y1}"), Kind::Ratio, quantile(&size, 0.5));
    b.num(format!("works_size_p90_{y1}"), Kind::Ratio, quantile(&size, 0.9));
    b.num(format!("works_size_largest_{y1}"), Kind::Ratio, quantile(&size, 1.0));
    // capacity, built and on order, at the year's capital-goods prices in the works' town
    // (what families::works_value counts)
    let p = if st.pbar.rows > 0 { &st.pbar } else { &st.price };
    let mut value: Vec<f64> = alive
        .iter()
        .map(|&f| {
            let (m, i) = (st.f_method[f], st.f_town[f]);
            (st.f_K[f] + st.f_order_K[f]) * (0..C.G).map(|g| C.m_cap[(m, g)] * p[(i, g)]).sum::<f64>()
        })
        .collect();
    value.sort_by(|a, b| b.total_cmp(a));
    let top = value.len().div_ceil(100);
    let total: f64 = value.iter().sum();
    b.num(format!("works_top_1pct_capacity_share_{y1}"), Kind::Share, if total > 0.0 { value[..top].iter().sum::<f64>() / total } else { f64::NAN });
    let mut life: Vec<f64> =
        (0..st.F_count).filter(|&f| !st.f_alive[f] && st.f_died[f] >= y0).map(|f| (st.f_died[f] - st.f_born[f]) as f64).collect();
    life.sort_by(f64::total_cmp);
    b.num("works_closed".into(), Kind::Count, life.len() as f64);
    b.num("works_closed_median_life".into(), Kind::Real, quantile(&life, 0.5));
    let util: Vec<f64> = alive.iter().filter(|&&f| st.f_K[f] > 0.0).map(|&f| st.f_u[f]).collect();
    b.num(format!("works_utilisation_{y1}"), Kind::Share, mean(&util));

    // ---- land
    b.group = GROUPS[4];
    // a town, a works' yard or dereliction takes a cell or leaves it: counted in whole cells
    let N = rec.N;
    let layer = rec.layer("use");
    let year_use = |k: usize| &layer[k * N..(k + 1) * N];
    for (name, code) in [("urban", USE_URBAN), ("industry", USE_INDUSTRY), ("derelict", USE_DERELICT)] {
        for &y in &marks {
            let cells = year_use((y - y0) as usize).iter().filter(|&&u| u == code).count();
            b.num(format!("cells_{name}_{y}"), Kind::Count, cells as f64);
        }
    }
    // the rest of the land is shared out within each cell, so it is counted by area: every
    // cell's share summed, in cells. (By main use a wood that is a third of every cell around
    // it would count for nothing.) Farms, pastures and woods under their own names; quarries,
    // pits and mines together; then what is left wild.
    let surface = |s: usize| matches!(C.sector_field[s].as_str(), "fertility" | "pasture" | "forest");
    let mut areas: Vec<(String, Vec<usize>)> = (0..C.NS).filter(|&s| surface(s)).map(|s| (C.sector_name[s].clone(), vec![s])).collect();
    areas.push(("mines".into(), (0..C.NS).filter(|&s| !surface(s)).collect()));
    areas.push(("wild".into(), vec![C.NS]));
    for (name, rows) in &areas {
        for &y in &marks {
            let area = g.area.get((y - y0) as usize).map_or(f64::NAN, |a| rows.iter().map(|&r| a[r]).sum());
            b.num(format!("area_{name}_{y}"), Kind::Real, area);
        }
    }
    // of the cells urban in a year, the share not urban ten years on
    let mut churn = Vec::new();
    for k in 0..n.saturating_sub(10) {
        let (now, later) = (year_use(k), year_use(k + 10));
        let urban = now.iter().filter(|&&u| u == USE_URBAN).count();
        if urban > 0 {
            let gone = (0..N).filter(|&c| now[c] == USE_URBAN && later[c] != USE_URBAN).count();
            churn.push(gone as f64 / urban as f64);
        }
    }
    b.num("urban_churn_10y".into(), Kind::Share, mean(&churn));

    // ---- methods and inventions
    b.group = GROUPS[5];
    // all output of the good a method makes, by any method, year by year
    let good_total = |m: usize| -> Vec<f64> {
        let mut tot = vec![0.0; n];
        for m2 in (0..C.M).filter(|&m2| C.method_out[m2] == C.method_out[m]) {
            for (k, v) in series(&format!("mout_{}", C.method_name[m2])).iter().enumerate() {
                tot[k] += if v.is_finite() { *v } else { 0.0 };
            }
        }
        tot
    };
    let mut never_built = Vec::new();
    for j in (0..C.NT).filter(|&j| !C.tech_known[j]) {
        let name = &C.tech_name[j];
        b.year(format!("discovered_{name}"), Some(st.tech_year[j]).filter(|&y| y >= y0));
        if j >= C.M {
            continue; // a tech nobody builds works for
        }
        let out = series(&format!("mout_{name}"));
        let first = out.iter().position(|&v| v > 0.0);
        b.year(format!("first_output_{name}"), first.map(|k| years[k]));
        let tot = good_total(j);
        b.year(format!("half_of_output_{name}"), (0..out.len()).find(|&k| tot[k] > 0.0 && out[k] >= 0.5 * tot[k]).map(|k| years[k]));
        if first.is_none() {
            never_built.push(name.clone());
        }
    }
    for (name, y) in METHOD_SHARES {
        let Some(m) = C.method_name.iter().position(|x| x == name) else { continue };
        if y < y0 || y > y1 {
            continue;
        }
        let k = (y - y0) as usize;
        let tot = good_total(m)[k];
        b.num(format!("share_{name}_{y}"), Kind::Share, if tot > 0.0 { at(&format!("mout_{name}"), y) / tot } else { f64::NAN });
    }

    // ---- prices and cycles
    b.group = GROUPS[6];
    let from = if y1 >= PRICE_FROM + 2 { PRICE_FROM.max(y0) } else { y0 };
    for good in PRICE_GOODS {
        if !C.good_index.contains_key(good) {
            continue;
        }
        let price = series(&format!("price_{good}")).get((from - y0) as usize..).unwrap_or(&[]);
        let d: Vec<f64> = price.windows(2).map(|w| (w[1] / w[0]).ln()).filter(|v| v.is_finite()).collect();
        let m = mean(&d);
        let sd = if d.len() >= 2 { mean(&d.iter().map(|v| (v - m) * (v - m)).collect::<Vec<_>>()).sqrt() } else { f64::NAN };
        b.num(format!("price_swing_{good}"), Kind::Real, sd);
    }
    let farm = series("ag_share");
    let zig: Vec<f64> = farm.windows(3).map(|w| (w[2] - 2.0 * w[1] + w[0]).abs()).filter(|v| v.is_finite()).collect();
    b.num("farm_zigzag".into(), Kind::Real, 100.0 * mean(&zig));

    // ---- news: the sweep adds one fact per kind, once it knows every kind any run had
    let mut news = BTreeMap::new();
    for e in &r.events {
        *news.entry(e.kind.clone()).or_insert(0) += 1;
    }

    // ---- speed
    b.group = SPEED;
    b.num("seconds".into(), Kind::Real, seconds);

    Facts { facts: b.facts, never_built, news }
}
