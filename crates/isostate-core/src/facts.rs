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
pub const GROUPS: [&str; 10] = [
    "People and towns",
    "Work and wages",
    "Railways",
    "Works",
    "Land",
    "Methods and inventions",
    "Prices and cycles",
    "News",
    "Speed",
    // canals (docs/rules/canals.md): at the end, so that the others keep their places
    "Canals",
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

/// The good whose dearest year is set against its middling one (a famine of tools).
pub const TOOLS: &str = "tools";

/// The first ten years are left out of the price swings: prices are still settling.
pub const PRICE_FROM: i64 = 1760;

/// Timber, whose swing is the building cycle's (docs/probes/tr-cycles.md), and the
/// year its second swing is read from: the pass over the rules reads swings from 1800
/// (docs/designs/rules-pass.md, aim 8 and section 8).
pub const TIMBER: &str = "timber";
pub const TIMBER_FROM: i64 = 1800;

/// The years about each year that its tools' price is set against (aim 11): itself and
/// twelve on either side, fewer at the run's ends.
pub const TOOLS_WINDOW: usize = 25;

/// How many of a yearly series' values are over twice the median of the `window`
/// years centred on them (cut short at the series' ends).
pub fn years_over_twice_local_median(p: &[f64], window: usize) -> usize {
    let half = window / 2;
    (0..p.len())
        .filter(|&k| {
            let mut near: Vec<f64> = p[k.saturating_sub(half)..(k + half + 1).min(p.len())].iter().copied().filter(|v| v.is_finite()).collect();
            near.sort_by(f64::total_cmp);
            p[k] > 2.0 * quantile(&near, 0.5)
        })
        .count()
}

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
    /// Every town's rate of natural increase in every year (people: docs/rules/people.md).
    pub rates: Vec<f64>,
    /// Per year: the workings of each extractive sector (mines: docs/rules/mines.md).
    pub mines: Vec<crate::mines::YearNote>,
    /// Per year: of the works' hands at the year's end, the share whose works stand
    /// within a path cost of 2 of their market (`works.walk`; docs/rules/works.md).
    pub near_market: Vec<f64>,
    /// What only a month can show. Empty unless the run was stepped a month at a
    /// time with `gather_month` (`run_year_noting` does it).
    pub months: MonthNotes,
}

/// The famine line of `people::growth`: a town that got under this share of the
/// food it needs in a month went hungry that month.
pub const FAMINE_LINE: f64 = 0.82;

/// A town paying more than this many times the middle town's wage is a wage gone
/// wild (docs/designs/rules-pass.md, aim 13; `examples/wage_watch.rs`).
pub const WILD_WAGE: f64 = 10.0;

/// What the sweep notes month by month for the pass over the rules
/// (docs/designs/rules-pass.md, section 5, "Stage 1's facts"), because nothing in
/// a run keeps it. Town-months count the towns standing with people in them.
#[derive(Clone, Debug, Default)]
pub struct MonthNotes {
    pub town_months: usize,
    /// Town-months whose wage was over `WILD_WAGE` times the middle town's (the
    /// median of the towns' wages that month, each town once, as `wage_watch` has it).
    pub wage_wild: usize,
    /// The greatest a town's wage ever was against the middle town's.
    pub wage_greatest_vs_middle: f64,
    /// Town-months with more at work than the town's people of working age
    /// (`month_stats.employment` over `participation` times the people, as
    /// `examples/overhire.rs` counts them).
    pub more_at_work_than_people: usize,
    /// Town-months from February to December with a wage the month before, and of
    /// them those whose wage moved by half or more (`carting.adjust`).
    pub wage_compared: usize,
    pub wage_moved_by_half: usize,
    /// Every town's wage last month.
    pub last_wage: Vec<f64>,
    /// Per month: hands in quarries, clay pits and mines over all at work.
    pub diggings: Vec<f64>,
    /// Per year: the weight carried times the path cost it was carried over (each
    /// cell's produce and works' goods to their market, and the trade between
    /// towns), at the year's path costs.
    pub weight_path: Vec<f64>,
    /// Each cell's market in December, to see which towns took cells at the year's end.
    pub catch_december: Vec<i64>,
    /// The towns that took cells from another town's market at the last year's end.
    pub took_cells: Vec<bool>,
    /// January and February town-months under the famine line in a town that took
    /// cells at the year's end (aim 12; docs/probes/tr-carriage.md, (d)).
    pub hungry_after_new_cells: usize,
}

/// Twelve months, each noted, then the year's end, noted: `sim::run_year` with the
/// facts' gathering in between. The sweep runs every year this way; nothing it
/// notes changes the run.
pub fn run_year_noting(sim: &mut Sim, g: &mut Gathered) -> Result<(), String> {
    for month in 0..12 {
        crate::sim::run_month(sim, month)?;
        gather_month(g, sim, month);
    }
    crate::sim::end_year(sim);
    gather_year(g, sim);
    Ok(())
}

/// Call after each `sim::run_month`, before the year's end.
pub fn gather_month(g: &mut Gathered, sim: &Sim, month: usize) {
    let (st, P) = (&sim.st, &sim.params);
    let n = &mut g.months;
    let ms = &st.month_stats;
    let act: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t] && st.pop[t] > 0.0).collect();
    let mut wages: Vec<f64> = act.iter().map(|&t| st.wage[t]).collect();
    wages.sort_by(f64::total_cmp);
    let middle = wages.get(wages.len() / 2).copied().unwrap_or(f64::NAN);
    for &t in &act {
        n.town_months += 1;
        let rel = st.wage[t] / middle;
        n.wage_wild += (rel > WILD_WAGE) as usize;
        if rel > n.wage_greatest_vs_middle {
            n.wage_greatest_vs_middle = rel;
        }
        n.more_at_work_than_people += ms.employment.get(t).is_some_and(|&e| e > P.participation * st.pop[t] * (1.0 + 1e-9)) as usize;
        // January begins on the year's new districts, so it is not set against December
        if month > 0 && n.last_wage.get(t).is_some_and(|&w| w > 0.0) {
            n.wage_compared += 1;
            n.wage_moved_by_half += ((st.wage[t] / n.last_wage[t]).ln().abs() >= 1.5f64.ln()) as usize;
        }
        if month < 2 && n.took_cells.get(t) == Some(&true) && st.food_ratio[t] < FAMINE_LINE {
            n.hungry_after_new_cells += 1;
        }
    }
    n.last_wage = st.wage.clone();
    // the observer's land is the first three sectors (farm, pasture, woods); the rest dig
    let all: f64 = ms.employment.iter().sum();
    let dig: f64 = ms.ex_labor_by_sector.iter().skip(3).sum();
    n.diggings.push(if all > 0.0 { dig / all } else { f64::NAN });
    if month == 11 {
        // the year's flows, summed over its months until the year's end clears them
        let land: f64 = (0..st.N).filter(|&c| st.catch[c] >= 0 && st.dcatch[c].is_finite()).map(|c| st.cell_flow[c] * st.dcatch[c]).sum();
        let mut trade = 0.0;
        for i in (0..st.T_max).filter(|&i| st.town_active[i]) {
            for j in (0..st.T_max).filter(|&j| j != i && st.town_active[j]) {
                let c = st.town_cost[(i, j)];
                if c.is_finite() {
                    trade += st.trade_F[(i, j)] * c;
                }
            }
        }
        n.weight_path.push(land + trade);
        n.catch_december = st.catch.clone();
    }
}

/// Call after each `sim::run_year` (or `sim::end_year`).
pub fn gather_year(g: &mut Gathered, sim: &Sim) {
    let (st, P) = (&sim.st, &sim.params);
    // the towns that took cells from another town's market at this year's end
    if g.months.catch_december.len() == st.N {
        let mut took = vec![false; st.T_max];
        for (c, &was) in g.months.catch_december.iter().enumerate() {
            let now = st.catch[c];
            if was >= 0 && now >= 0 && now != was {
                took[now as usize] = true;
            }
        }
        g.months.took_cells = took;
    }
    // works.walk: the works' hands near their market (examples/works_census.rs, "near")
    let (mut near, mut hands) = (0.0, 0.0);
    for f in (0..st.f_alive.len().min(st.f_labor.len())).filter(|&f| st.f_alive[f]) {
        hands += st.f_labor[f];
        if st.dcatch[st.f_cell[f]] <= 2.0 {
            near += st.f_labor[f];
        }
    }
    g.near_market.push(if hands > 0.0 { near / hands } else { f64::NAN });
    let (mut capped, mut all) = (0.0, 0.0);
    for t in 0..st.T_max {
        if !st.town_active[t] || st.y_ratio[t] <= 0.0 {
            continue;
        }
        // the rule in people::growth before its clamp (a famine's two points off are not seen here)
        let rate = P.growth_slope * st.y_ratio[t].max(1e-3).ln() + P.growth_base;
        all += st.pop[t];
        // people: under the game's rule nothing is clamped; a town counts as held when
        // income has carried its rate to within a twentieth of the rule's ceiling
        let held = if P.people.enabled { st.pp_income_term[t] >= 0.95 * P.people.growth_top } else { rate >= P.growth_cap };
        if held {
            capped += st.pop[t];
        }
        g.rates.push(if P.people.enabled { st.pp_growth[t] } else { rate.clamp(-0.04, P.growth_cap) });
    }
    g.capped.push(if all > 0.0 { capped / all } else { f64::NAN });
    g.area.push((0..st.lam.rows).map(|r| (0..st.N).map(|c| st.lam[(r, c)]).sum()).collect());
    g.mines.push(crate::mines::gather(st, &sim.content, P));
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

/// How many years a mean over years spans (R11 in the backlog's sweep of findings).
/// A single year's real wage moves by more than most changes to the rules do: it
/// read 0.2 lower on every island of eight and 0.05 higher on the next eight
/// (`docs/rules/towns.md`). Twenty years take out the weather and the building
/// cycle and keep the century's shape.
pub const SPAN: i64 = 20;

/// The spans of years the means are taken over, as (first, last): twenty years
/// ending twenty, forty, sixty years after the first year, and so on, then the
/// twenty ending in the run's last year if that is not already one of them.
/// 1751-1770, 1771-1790 ... 1871-1890 and 1881-1900 for a whole run; none for a
/// run of under twenty years after its first.
pub fn spans(y0: i64, y1: i64) -> Vec<(i64, i64)> {
    let mut v = Vec::new();
    let mut end = y0 + SPAN;
    while end <= y1 {
        v.push((end - SPAN + 1, end));
        end += SPAN;
    }
    if y1 - y0 >= SPAN && v.last().map(|s| s.1) != Some(y1) {
        v.push((y1 - SPAN + 1, y1));
    }
    v
}

/// The decades of a run as (its name, first year, last year): 1750 for 1750-1759 and so
/// on, the last taking in the run's last year (1890 for 1890-1900), as
/// `examples/trace_works.rs` takes them. The first begins with the run.
pub fn decades(y0: i64, y1: i64) -> Vec<(i64, i64, i64)> {
    let mut v = Vec::new();
    let mut d = y0.div_euclid(10) * 10;
    while d <= y1 {
        let end = if d + 19 > y1 { y1 } else { d + 9 };
        v.push((d, d.max(y0), end));
        if end == y1 {
            break;
        }
        d += 10;
    }
    v
}

/// The mean of a yearly series over the years `a..=b`, leaving out years with no
/// value (a share whose total was empty all year); NaN if none has one.
fn span_mean(v: &[f64], y0: i64, a: i64, b: i64) -> f64 {
    let vals: Vec<f64> = (a..=b).filter_map(|y| v.get((y - y0) as usize).copied()).filter(|x| x.is_finite()).collect();
    mean(&vals)
}

/// Every observer series averaged over each year's twelve months: `[series][year]`,
/// in the registry's order. NaN where a share had an empty total all year.
pub fn annual_means(r: &RunResult) -> Vec<Vec<f64>> {
    let y0 = r.content.start_year;
    r.state.series.data.iter().map(|d| (0..r.rec.n_years).map(|k| annual(d, y0, y0 + k as i64)).collect()).collect()
}

/// Each year's wage of the median town by people: the year's mean wage of the town
/// in which, the towns taken from the lowest wage up, half the island's people have
/// been counted. From the yearly snapshot; NaN for a year with nobody in a town.
pub fn median_town_wage(rec: &crate::record::Record) -> Vec<f64> {
    let T = rec.T;
    (0..rec.n_years)
        .map(|k| {
            let mut towns: Vec<(f64, f64)> = (0..T)
                .filter(|&t| rec.town_active[k * T + t] == 1 && rec.town_pop[k * T + t] > 0.0)
                .map(|t| (rec.town_wage[k * T + t] as f64, rec.town_pop[k * T + t] as f64))
                .collect();
            towns.sort_by(|a, b| a.0.total_cmp(&b.0));
            let half = 0.5 * towns.iter().map(|&(_, p)| p).sum::<f64>();
            let mut seen = 0.0;
            towns.iter().find(|&&(_, p)| {
                seen += p;
                seen >= half
            }).map_or(f64::NAN, |&(w, _)| w)
        })
        .collect()
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

/// How many years before a year's own harvest still count as its weather in
/// `farm_zigzag_beyond_weather`. The harvest page finds the farm hands answering a
/// short crop for two to four years after it, each echo about half the last.
pub const WEATHER_LAGS: usize = 4;

/// Least squares with a constant: the coefficients `[constant, x0, x1, ...]` that
/// fit `y` best from the rows of `x`. None when the columns do not tell the
/// coefficients apart (a harvest the same every year).
fn least_squares(x: &[Vec<f64>], y: &[f64]) -> Option<Vec<f64>> {
    let p = x.first()?.len() + 1;
    if x.len() < p {
        return None;
    }
    // the normal equations, then elimination with the largest pivot
    let mut a = vec![vec![0.0; p + 1]; p];
    for (row, &target) in x.iter().zip(y) {
        let full: Vec<f64> = std::iter::once(1.0).chain(row.iter().copied()).collect();
        for i in 0..p {
            for j in 0..p {
                a[i][j] += full[i] * full[j];
            }
            a[i][p] += full[i] * target;
        }
    }
    let scale = (0..p).map(|i| a[i][i].abs()).fold(0.0, f64::max);
    for c in 0..p {
        let pivot = (c..p).max_by(|&i, &j| a[i][c].abs().total_cmp(&a[j][c].abs()))?;
        if a[pivot][c].abs() <= 1e-12 * scale {
            return None;
        }
        a.swap(c, pivot);
        for i in (0..p).filter(|&i| i != c) {
            let f = a[i][c] / a[c][c];
            for j in c..=p {
                a[i][j] -= f * a[c][j];
            }
        }
    }
    Some((0..p).map(|i| a[i][p] / a[i][i]).collect())
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

    /// A fact already made, again under a name that says its place in the run and
    /// not its year (`pop_last` for `pop_1900`), under the current heading. Nothing
    /// if there is no such fact (a run too short for a twenty-year mean).
    fn again(&mut self, from: &str, name: &str) {
        if let Some(f) = self.facts.iter().find(|f| f.name == from) {
            let f = Fact { name: name.to_string(), group: self.group, ..f.clone() };
            self.facts.push(f);
        }
    }
}

/// The ages at which the chance of a works closing is read, in years, both ends in
/// (`examples/trace_works.rs`).
pub const CLOSING_AGES: [(i64, i64, &str); 7] =
    [(0, 2, "0_2"), (3, 5, "3_5"), (6, 9, "6_9"), (10, 14, "10_14"), (15, 24, "15_24"), (25, 49, "25_49"), (50, i64::MAX, "50_up")];

/// Works-years lived at an age from `a` to `z` between `y0` and `y1`, and the works
/// that closed at such an age, of every works the state has ever had.
pub fn closing_at_age(st: &crate::state::State, y0: i64, y1: i64, a: i64, z: i64) -> (f64, f64) {
    let (mut at_risk, mut closed) = (0.0, 0.0);
    for f in 0..st.F_count {
        let born = st.f_born[f];
        let end = if st.f_alive[f] { y1 } else { st.f_died[f] };
        for y in born.max(y0)..=end.min(y1) {
            if (a..=z).contains(&(y - born)) {
                at_risk += 1.0;
                closed += (!st.f_alive[f] && st.f_died[f] == y) as u8 as f64;
            }
        }
    }
    (at_risk, closed)
}

/// The middle year of a run, for the facts named `_mid`: 1825 for a run from 1750 to
/// 1900, 1900 for a run to 2050.
pub fn middle(y0: i64, y1: i64) -> i64 {
    y0 + (y1 - y0) / 2
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
    // people: how the towns' rates of natural increase are spread, in per cent a year
    let mut rates = g.rates.clone();
    rates.sort_by(f64::total_cmp);
    for (name, q) in [("p10", 0.1), ("p50", 0.5), ("p90", 0.9)] {
        b.num(format!("town_growth_pct_{name}"), Kind::Ratio, 100.0 * quantile(&rates, q));
    }
    // and the share of town-years within a tenth of a point of the port's cap
    let r_cap = r.params.growth_cap;
    let near = rates.iter().filter(|&&r| (r - r_cap).abs() <= 0.001).count();
    b.num("town_growth_near_old_cap".into(), Kind::Share, if rates.is_empty() { f64::NAN } else { near as f64 / rates.len() as f64 });
    for &y in &marks {
        b.num(format!("urban_share_{y}"), Kind::Share, at("urban_share", y));
    }
    // twenty-year means (SPAN): urban_share_1881_1900 is the mean of the twenty years' yearly means
    for &(a, z) in &spans(y0, y1) {
        b.num(format!("urban_share_{a}_{z}"), Kind::Share, span_mean(series("urban_share"), y0, a, z));
    }
    for &y in &marks {
        b.num(format!("towns_{y}"), Kind::Count, at("towns", y));
    }
    // The same by their place in the run and not their year (R7): the first year, the
    // middle, the last and the last twenty years, so that sweeps to different end years
    // have names in common (docs/rules/scorecard.md says which to read when).
    let mid = middle(y0, y1);
    let last20 = spans(y0, y1).last().copied();
    let placed = |name: &str| -> [String; 3] {
        let (a, z) = last20.unwrap_or((y1, y1));
        [format!("{name}_{y0}"), format!("{name}_{y1}"), if last20.is_some() { format!("{name}_{a}_{z}") } else { String::new() }]
    };
    b.again(&format!("pop_{y1}"), "pop_last");
    b.num("pop_mid".into(), Kind::Count, at("pop", mid));
    let [first, last, last_span] = placed("urban_share");
    b.again(&first, "urban_share_first");
    b.num("urban_share_mid".into(), Kind::Share, at("urban_share", mid));
    b.again(&last, "urban_share_last");
    b.again(&last_span, "urban_share_last20");
    b.again(&format!("towns_{y1}"), "towns_last");
    // the pass over the rules (docs/designs/rules-pass.md, aims 1 and 4): how many times the
    // first year's people the last year has, and whether the towns' share is still rising
    // at the end: the last twenty years' mean less the twenty years' before them
    b.num("pop_last_vs_first".into(), Kind::Ratio, at("pop", y1) / at("pop", y0));
    if let Some((a, z)) = last20.filter(|&(a, _)| a - SPAN > y0) {
        let u = series("urban_share");
        b.num("urban_share_last20_gain".into(), Kind::Real, span_mean(u, y0, a, z) - span_mean(u, y0, a - SPAN, z - SPAN));
    }
    let last_town = (0..st.T_max).filter(|&t| st.town_active[t] && st.town_founded[t] >= y0).map(|t| st.town_founded[t]).max();
    b.year("last_town_founded".into(), last_town);
    // people: the towns founded from 1800 on, by what their districts were known for
    // (a town can also be a port or on a railway), and how big they are at the end
    if r.params.people.enabled {
        let late: Vec<&crate::peopling::Founding> = st.pp_foundings.iter().filter(|f| f.year >= 1800).collect();
        let count = |f: &dyn Fn(&crate::peopling::Founding) -> bool| late.iter().filter(|x| f(x)).count() as f64;
        b.num("late_towns".into(), Kind::Count, late.len() as f64);
        for kind in ["coal", "iron", "stone", "works", "farm"] {
            b.num(format!("late_towns_{kind}"), Kind::Count, count(&|f| f.kind == kind));
        }
        b.num("late_towns_port".into(), Kind::Count, count(&|f| f.port));
        b.num("late_towns_rail".into(), Kind::Count, count(&|f| f.rail));
        let mut size: Vec<f64> = late.iter().map(|f| st.pop[f.town]).collect();
        size.sort_by(f64::total_cmp);
        b.num("late_town_people_median".into(), Kind::Count, quantile(&size, 0.5));
        b.num("late_town_people_largest".into(), Kind::Count, size.last().copied().unwrap_or(f64::NAN));
        b.num("emigrants".into(), Kind::Count, st.pp_emigrants);
    }

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
    // carting (docs/rules/carting.md): living standards in units that compare across
    // runs, and the carting bill
    for (name, kind, v) in crate::carting::facts(r) {
        b.num(name, kind, v);
    }
    // accounts (docs/rules/accounts.md): who earns what, only of a run that kept books
    for (name, kind, v) in crate::accounts::facts(r) {
        b.num(name, kind, v);
    }
    // twenty-year means (SPAN) of what a scorecard reads: what a wage buys and output per
    // head against the first year (real_wage_1881_1900_vs_1750), the shares of work, and
    // work's share of the island's income where the books are kept
    let spans = spans(y0, y1);
    let mut against_first = vec![("real_wage", "real_wage"), ("output_per_head", "gdp_pc")];
    if st.series.index.contains_key("real_wage_chained") {
        against_first.push(("real_wage_chained", "real_wage_chained"));
    }
    for (label, name) in against_first {
        for &(a, z) in &spans {
            b.num(format!("{label}_{a}_{z}_vs_{y0}"), Kind::Ratio, span_mean(series(name), y0, a, z) / at(name, y0));
        }
    }
    let (ag, ind) = (series("ag_share"), series("ind_share"));
    let mines: Vec<f64> = ag.iter().zip(ind).map(|(a, i)| 1.0 - a - i).collect();
    for (label, v) in [("farm_share", ag), ("industry_share", ind), ("mine_share", &mines[..])] {
        for &(a, z) in &spans {
            b.num(format!("{label}_{a}_{z}"), Kind::Share, span_mean(v, y0, a, z));
        }
    }
    if st.series.index.contains_key("acct_share_work") {
        for &(a, z) in &spans {
            b.num(format!("income_share_work_{a}_{z}"), Kind::Share, span_mean(series("acct_share_work"), y0, a, z));
        }
    }
    // by their place in the run (R7), as for the towns above
    for name in ["farm_share", "industry_share", "mine_share"] {
        let [first, last, last_span] = placed(name);
        b.again(&first, &format!("{name}_first"));
        b.again(&last, &format!("{name}_last"));
        b.again(&last_span, &format!("{name}_last20"));
    }
    for (label, name) in [("real_wage", "real_wage"), ("output_per_head", "gdp_pc"), ("real_wage_chained", "real_wage_chained")] {
        if !st.series.index.contains_key(name) {
            continue;
        }
        let [_, last, last_span] = placed(label);
        if y1 > y0 {
            b.num(format!("{label}_mid_vs_first"), Kind::Ratio, at(name, mid) / at(name, y0));
        }
        b.again(&format!("{last}_vs_{y0}"), &format!("{label}_last_vs_first"));
        b.again(&format!("{last_span}_vs_{y0}"), &format!("{label}_last20_vs_first"));
    }
    b.again(&placed("income_share_work")[2], "income_share_work_last20");

    // ---- the pass over the rules, stage 1 (docs/designs/rules-pass.md, section 5): the
    // facts the card needs that the sweep did not have
    // The diggings' own share of hands. `mine_share` is one less the land's and the works'
    // shares, so under carting it counts the carters (and the navvies and menders) as well;
    // this counts only the quarries, clay pits and mines, from the month's own hands.
    let diggings: Vec<f64> = if g.months.diggings.len() == 12 * n { g.months.diggings.chunks(12).map(mean).collect() } else { vec![f64::NAN; n] };
    for &y in &marks {
        b.num(format!("diggings_share_{y}"), Kind::Share, diggings[(y - y0) as usize]);
    }
    for &(a, z) in &spans {
        b.num(format!("diggings_share_{a}_{z}"), Kind::Share, span_mean(&diggings, y0, a, z));
    }
    // Spending a head over the chained cost of living (docs/probes/tr-works.md, section 1):
    // `gdp_nom / pop / cpi_chained`, by decade against the first, its highest year, and the
    // last decade against the one twenty years before it (aim 6: the 1890s over the 1870s).
    // The observer's `gdp_pc` is deflated by the basket of 1750, which turns a stall into a fall.
    if st.series.index.contains_key("cpi_chained") {
        let (nom, pop, cpi) = (series("gdp_nom"), series("pop"), series("cpi_chained"));
        let spend: Vec<f64> = (0..n).map(|k| nom[k] / pop[k] / cpi[k]).collect();
        let ds = decades(y0, y1);
        let dmean = |&(_, a, z): &(i64, i64, i64)| span_mean(&spend, y0, a, z);
        for d in &ds[1..] {
            b.num(format!("spending_per_head_chained_{}s_vs_{}s", d.0, ds[0].0), Kind::Ratio, dmean(d) / dmean(&ds[0]));
        }
        if let (Some(last), true) = (ds.last(), ds.len() > 2) {
            let before = &ds[ds.len() - 3];
            b.num(format!("spending_per_head_chained_{}s_vs_{}s", last.0, before.0), Kind::Ratio, dmean(last) / dmean(before));
        }
        let peak = (0..n).filter(|&k| spend[k].is_finite()).max_by(|&i, &j| spend[i].total_cmp(&spend[j]));
        b.year("spending_per_head_chained_peak_year".into(), peak.map(|k| years[k]));
    }
    // Carters between towns (aim 19): each town's carters times the share of its carting
    // bill that is the works' carting and the trade it sends, the rest being its land's
    // produce on the way to market; over the hands, as `carters_share_of_hands_Y` has them.
    if st.acct.on && r.params.carting.enabled {
        use crate::accounts::{KT, TOWN_FIELDS};
        let col = |name: &str| TOWN_FIELDS.iter().position(|&c| c == name).expect("a column of the books");
        let (carters, land, works, trade) = (col("carters"), col("cart_land"), col("cart_works"), col("cart_trade"));
        for &y in &marks {
            let book = st.acct.years.iter().position(|yr| yr.year == y).and_then(|k| st.acct.year(k));
            let between = book.map_or(f64::NAN, |rec| {
                let mut v = 0.0;
                for t in 0..st.T_max {
                    let row = &rec.towns[t * KT..(t + 1) * KT];
                    let bill = (row[land] + row[works] + row[trade]) as f64;
                    if bill > 0.0 {
                        v += row[carters] as f64 * (row[works] + row[trade]) as f64 / bill;
                    }
                }
                v
            });
            b.num(format!("carters_between_towns_share_of_hands_{y}"), Kind::Share, between / (r.params.participation * at("pop", y)));
        }
    }
    // The carters a unit of weight and path cost (design 4.6, "How it is checked"): the
    // year's carters (the mean of its months) over the year's weight carried times the path
    // cost it went over, per thousand. Under a fixed freight it falls as the wage rises.
    if r.params.carting.enabled && g.months.weight_path.len() == n {
        let warm = r.params.warmup_years.max(0) as usize;
        let per = |y: i64| -> f64 {
            let k = (y - y0) as usize;
            st.cart_years.get(warm + k).map_or(f64::NAN, |&(_, c)| 1000.0 * c / 12.0 / g.months.weight_path[k])
        };
        for &y in &marks {
            b.num(format!("carters_per_1000_weight_path_{y}"), Kind::Real, per(y));
        }
        for &y in &marks[1..] {
            b.num(format!("carters_per_1000_weight_path_{y}_vs_{y0}"), Kind::Ratio, per(y) / per(y0));
        }
    }
    // Town-months (aim 13 and the faults of section 4.1; `carting.adjust`), from the months
    // the sweep noted; no value for a run stepped a year at a time.
    let m = &g.months;
    let noted = |v: f64| if m.town_months > 0 { v } else { f64::NAN };
    b.num("town_months_wage_over_ten_middle".into(), Kind::Count, noted(m.wage_wild as f64));
    b.num("town_wage_greatest_vs_middle".into(), Kind::Ratio, noted(m.wage_greatest_vs_middle));
    b.num("town_months_more_at_work_than_people".into(), Kind::Count, noted(m.more_at_work_than_people as f64));
    b.num("town_months_wage_moved_by_half".into(), Kind::Share, if m.wage_compared > 0 { m.wage_moved_by_half as f64 / m.wage_compared as f64 } else { f64::NAN });

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
    // by their place in the run (R7)
    b.num("rail_km_mid".into(), Kind::Count, at("rail_km", mid));
    b.again(&format!("rail_km_{y1}"), "rail_km_last");
    // railways (docs/rules/railways.md): the network by decade, what it serves and what it cost
    for (name, kind, v) in crate::railways::facts(r) {
        b.num(name, kind, v);
    }
    // canals (docs/rules/canals.md): the network, what it carries, and coal's price
    // and uses, which any run can answer
    b.group = GROUPS[9];
    for (name, kind, v) in crate::canals::facts(r) {
        b.num(name, kind, v);
    }

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
    // works: every works opened from the first recorded year on, sister works included
    b.num("works_founded".into(), Kind::Count, (0..st.F_count).filter(|&f| st.f_born[f] >= y0).count() as f64);
    b.num("works_closed".into(), Kind::Count, life.len() as f64);
    b.num("works_closed_median_life".into(), Kind::Real, quantile(&life, 0.5));
    // the pass over the rules: the chance a works closes in a year, by its age (closures at
    // that age over works-years at that age, of works standing at any time from the first
    // year), which docs/probes/tr-works.md, section 3, reads in place of the median life
    for (a, z, name) in CLOSING_AGES {
        let (at_risk, closed) = closing_at_age(st, y0, y1, a, z);
        b.num(format!("works_closing_chance_age_{name}"), Kind::Share, if at_risk > 0.0 { closed / at_risk } else { f64::NAN });
    }
    // and the works' hands near their market (works.walk), at the marks
    for &y in &marks {
        b.num(format!("works_hands_within_2_of_market_{y}"), Kind::Share, g.near_market.get((y - y0) as usize).copied().unwrap_or(f64::NAN));
    }
    let util: Vec<f64> = alive.iter().filter(|&&f| st.f_K[f] > 0.0).map(|&f| st.f_u[f]).collect();
    b.num(format!("works_utilisation_{y1}"), Kind::Share, mean(&util));
    // by their place in the run (R7)
    for name in ["works", "works_ex_services", "works_per_1000_people", "works_size_median", "works_size_p90", "works_size_largest"] {
        b.again(&format!("{name}_{y1}"), &format!("{name}_last"));
    }
    b.again(&format!("works_top_1pct_capacity_share_{y1}"), "works_top_1pct_capacity_share_last");
    b.again(&format!("works_utilisation_{y1}"), "works_utilisation_last");

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
        // by its place in the run (R7)
        b.again(&format!("cells_{name}_{y1}"), &format!("cells_{name}_last"));
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
    // mines (docs/rules/mines.md): the workings, their hands, what they raise and what they pay
    for (name, kind, v) in crate::mines::facts(r, g) {
        b.num(name, kind, v);
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
        let half = (0..out.len()).find(|&k| tot[k] > 0.0 && out[k] >= 0.5 * tot[k]).map(|k| years[k]);
        b.year(format!("half_of_output_{name}"), half);
        // how long from its discovery to half its good's output (the prices page asks that
        // coke iron take no more than ten years; docs/rules/scorecard.md)
        let found = Some(st.tech_year[j]).filter(|&y| y >= y0);
        b.num(format!("years_to_half_{name}"), Kind::Real, match (found, half) {
            (Some(d), Some(h)) => (h - d) as f64,
            _ => f64::NAN,
        });
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
    let swing = |good: &str, from: i64| -> f64 {
        let price = series(&format!("price_{good}")).get((from - y0) as usize..).unwrap_or(&[]);
        let d: Vec<f64> = price.windows(2).map(|w| (w[1] / w[0]).ln()).filter(|v| v.is_finite()).collect();
        let m = mean(&d);
        if d.len() >= 2 { mean(&d.iter().map(|v| (v - m) * (v - m)).collect::<Vec<_>>()).sqrt() } else { f64::NAN }
    };
    for good in PRICE_GOODS {
        if !C.good_index.contains_key(good) {
            continue;
        }
        b.num(format!("price_swing_{good}"), Kind::Real, swing(good, from));
    }
    // the pass over the rules (aim 8): timber's swing, the building cycle's mark
    // (docs/probes/tr-cycles.md), as its neighbours have theirs and from 1800 as the aim has it
    if C.good_index.contains_key(TIMBER) {
        b.num(format!("price_swing_{TIMBER}"), Kind::Real, swing(TIMBER, from));
        if y1 >= TIMBER_FROM + 2 {
            b.num(format!("price_swing_{TIMBER}_from_{TIMBER_FROM}"), Kind::Real, swing(TIMBER, TIMBER_FROM.max(y0)));
        }
    }
    // A famine of tools (C137; docs/rules/works.md): when farm machinery comes the engine
    // works want tools the smithies left cannot make. Tools' dearest year from PRICE_FROM
    // against the middling (median) year, which year it was, and how many years cost
    // more than twice the middling one. The price is the island's, in money: it hardly
    // drifts over the century (the cost of living ends between 0.9 and 1.6 of 1750's).
    if C.good_index.contains_key(TOOLS) {
        let p: Vec<f64> = series(&format!("price_{TOOLS}")).get((from - y0) as usize..).unwrap_or(&[]).to_vec();
        let mut sorted: Vec<f64> = p.iter().copied().filter(|v| v.is_finite()).collect();
        sorted.sort_by(f64::total_cmp);
        let middling = quantile(&sorted, 0.5);
        let dearest = (0..p.len()).filter(|&k| p[k].is_finite()).max_by(|&i, &j| p[i].total_cmp(&p[j]));
        b.num(format!("price_{TOOLS}_dearest_vs_median"), Kind::Ratio, dearest.map_or(f64::NAN, |k| p[k] / middling));
        b.year(format!("price_{TOOLS}_dearest_year"), dearest.map(|k| from + k as i64));
        b.num(format!("price_{TOOLS}_years_over_twice_median"), Kind::Count, p.iter().filter(|&&v| v > 2.0 * middling).count() as f64);
        // the pass over the rules (aim 11): against the middling price of the TOOLS_WINDOW
        // years about each year and not the century's, since a price that falls all century
        // (as tools' does with the production tree) would read as a famine at its start
        b.num(format!("price_{TOOLS}_years_over_twice_local_median"), Kind::Count, years_over_twice_local_median(&p, TOOLS_WINDOW) as f64);
    }
    let farm = series("ag_share");
    let zig: Vec<f64> = farm.windows(3).map(|w| (w[2] - 2.0 * w[1] + w[0]).abs()).filter(|v| v.is_finite()).collect();
    b.num("farm_zigzag".into(), Kind::Real, 100.0 * mean(&zig));
    // harvest (C126): with the weather on, the zigzag is mostly the weather's, and no
    // longer tells the false cycle the prices page cured from good years and bad. Here the
    // year's second difference of the farm share is set against the log of the country's
    // harvest in that year and the WEATHER_LAGS before it (least squares, with a
    // constant), and what the harvests do not account for is measured as the zigzag
    // is: farm_zigzag_beyond_weather. With no harvests there is nothing to take out, and
    // it is the zigzag itself. farm_zigzag_weather_share is the share of the zigzag's
    // variance the harvests account for.
    let h: Vec<f64> = r.rec.harvest.iter().map(|&v| (v as f64).ln()).collect();
    let (beyond, weather) = if h.len() == farm.len() && h.len() > 2 * (WEATHER_LAGS + 3) {
        let (mut x, mut y) = (Vec::new(), Vec::new());
        for k in WEATHER_LAGS.max(2)..farm.len() {
            let d2 = farm[k] - 2.0 * farm[k - 1] + farm[k - 2];
            if d2.is_finite() {
                x.push((0..=WEATHER_LAGS).map(|lag| h[k - lag]).collect::<Vec<f64>>());
                y.push(d2);
            }
        }
        match least_squares(&x, &y) {
            Some(fit) => {
                let resid: Vec<f64> = x.iter().zip(&y).map(|(row, d)| d - fit[0] - row.iter().zip(&fit[1..]).map(|(a, c)| a * c).sum::<f64>()).collect();
                let my = mean(&y);
                let total: f64 = y.iter().map(|d| (d - my) * (d - my)).sum();
                let left: f64 = resid.iter().map(|e| e * e).sum();
                (100.0 * mean(&resid.iter().map(|e| e.abs()).collect::<Vec<_>>()), if total > 0.0 { 1.0 - left / total } else { f64::NAN })
            }
            None => (f64::NAN, f64::NAN),
        }
    } else {
        (100.0 * mean(&zig), f64::NAN)
    };
    b.num("farm_zigzag_beyond_weather".into(), Kind::Real, beyond);
    b.num("farm_zigzag_weather_share".into(), Kind::Share, weather);
    // harvest: how grain moves from year to year, and what the worst years do.
    // Yearly changes of the log, from PRICE_FROM as above. A lag-one correlation
    // near -0.5 is what independent yearly shocks to a level give; near zero or
    // above, the level wanders; well below, it rings.
    if C.good_index.contains_key("grain") {
        let changes = |name: &str| -> Vec<f64> {
            let v = series(name).get((from - y0) as usize..).unwrap_or(&[]);
            v.windows(2).map(|w| (w[1] / w[0]).ln()).filter(|d| d.is_finite()).collect()
        };
        let sd = |d: &[f64]| {
            let m = mean(d);
            if d.len() >= 2 { mean(&d.iter().map(|v| (v - m) * (v - m)).collect::<Vec<_>>()).sqrt() } else { f64::NAN }
        };
        let ac1 = |d: &[f64]| {
            let m = mean(d);
            let var: f64 = d.iter().map(|v| (v - m) * (v - m)).sum();
            let cov: f64 = d.windows(2).map(|w| (w[0] - m) * (w[1] - m)).sum();
            if d.len() >= 3 && var > 0.0 { cov / var } else { f64::NAN }
        };
        let (dp, dq) = (changes("price_grain"), changes("out_grain"));
        b.num("output_swing_grain".into(), Kind::Real, sd(&dq));
        b.num("price_ac1_grain".into(), Kind::Real, ac1(&dp));
        b.num("output_ac1_grain".into(), Kind::Real, ac1(&dq));
        // the hardest single year for wage earners: the real wage's largest fall on the year before
        let dw = changes("real_wage");
        b.num("real_wage_worst_year".into(), Kind::Real, dw.iter().copied().fold(f64::NAN, f64::min).exp_m1());
        // the pass over the rules (aim 7): the mean of the five worst, a steadier reading of
        // the same tail than the single worst (docs/probes/tr-works.md, section 4)
        let mut falls: Vec<f64> = dw.iter().map(|d| d.exp_m1()).collect();
        falls.sort_by(f64::total_cmp);
        b.num("real_wage_five_worst_years_mean".into(), Kind::Real, if falls.len() >= 5 { mean(&falls[..5]) } else { f64::NAN });
        b.num("real_wage_swing".into(), Kind::Real, sd(&dw));
        // The same fall read from the middle of the island and not its mean. The island's
        // wage is every town's averaged by its people, so one town's wild wage coming back
        // down reads as a fall for everyone (seed 11 with the new land: 2.29 then 1.00, read
        // as -0.565). Here each year's wage is the median town's, by people (half the
        // island's people live in towns that paid less that year), over the year's cost of
        // living, and the fact is its largest fall on the year before.
        let middle_wage = median_town_wage(rec);
        let cpi = series("cpi");
        let real: Vec<f64> = middle_wage.iter().zip(cpi).map(|(w, c)| w / c).collect();
        let dm: Vec<f64> = real.get((from - y0) as usize..).unwrap_or(&[]).windows(2).map(|w| (w[1] / w[0]).ln()).filter(|d| d.is_finite()).collect();
        b.num("real_wage_worst_year_median_town".into(), Kind::Real, dm.iter().copied().fold(f64::NAN, f64::min).exp_m1());
        // With harvests on: how large they were, and what a year's harvest does that
        // same year. Each `_on_harvest` is the slope of the yearly change in a log on
        // the log of the country's harvest (1 is an average year), over every year
        // after the first: -1.5 for the price says a harvest ten per cent short
        // comes with grain fifteen per cent dearer than the year before. The
        // `_year_after_harvest` slopes are the same over two years, from the year
        // before the harvest to the year after it: what is left of it by then.
        let h: Vec<f64> = r.rec.harvest.iter().map(|&v| (v as f64).ln()).collect();
        if h.len() >= 10 && h.len() == series("out_grain").len() {
            let slope = |name: &str, after: usize| -> f64 {
                let v = series(name);
                let (x, d): (Vec<f64>, Vec<f64>) =
                    (1..h.len() - after).map(|k| (h[k], (v[k + after] / v[k - 1]).ln())).filter(|(_, d)| d.is_finite()).unzip();
                let (mx, md) = (mean(&x), mean(&d));
                let sxx: f64 = x.iter().map(|a| (a - mx) * (a - mx)).sum();
                let sxd: f64 = x.iter().zip(&d).map(|(a, c)| (a - mx) * (c - md)).sum();
                if sxx > 0.0 { sxd / sxx } else { f64::NAN }
            };
            b.num("harvest_sd".into(), Kind::Real, sd(&h));
            b.num("grain_output_on_harvest".into(), Kind::Real, slope("out_grain", 0));
            b.num("grain_price_on_harvest".into(), Kind::Real, slope("price_grain", 0));
            b.num("real_wage_on_harvest".into(), Kind::Real, slope("real_wage", 0));
            b.num("grain_price_year_after_harvest".into(), Kind::Real, slope("price_grain", 1));
            b.num("real_wage_year_after_harvest".into(), Kind::Real, slope("real_wage", 1));
        }
    }
    // years in which people::growth found at least one town short of food
    let mut hungry: Vec<i64> = r.events.iter().filter(|e| e.kind == "famine").map(|e| e.year.floor() as i64).collect();
    hungry.dedup();
    b.num("famine_years".into(), Kind::Count, hungry.len() as f64);
    // people (docs/rules/people.md): years in which a thousandth of the island took ship after
    // a hungry year. Until 12 November 2026 these were filed as famine news and counted above.
    let mut sailed: Vec<i64> = r.events.iter().filter(|e| e.kind == "emigration").map(|e| e.year.floor() as i64).collect();
    sailed.dedup();
    b.num("emigration_years".into(), Kind::Count, sailed.len() as f64);
    // the pass over the rules (aim 12): hungry months just after a town took cells from
    // another town's market at the year's end, when the shelves stay behind
    // (docs/probes/tr-carriage.md, (d)); no value for a run stepped a year at a time
    b.num("hungry_months_after_new_cells".into(), Kind::Count, if g.months.town_months > 0 { g.months.hungry_after_new_cells as f64 } else { f64::NAN });

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_spans_are_twenty_years_and_the_last_ends_with_the_run() {
        assert_eq!(spans(1750, 1900), vec![(1751, 1770), (1771, 1790), (1791, 1810), (1811, 1830), (1831, 1850), (1851, 1870), (1871, 1890), (1881, 1900)]);
        // a run that ends on a span's end has no extra one
        assert_eq!(spans(1750, 1790), vec![(1751, 1770), (1771, 1790)]);
        assert_eq!(spans(1750, 1785), vec![(1751, 1770), (1766, 1785)]);
        assert_eq!(spans(1750, 1770), vec![(1751, 1770)]);
        assert!(spans(1750, 1769).is_empty() && spans(1750, 1750).is_empty());
        assert_eq!(spans(1750, 2050).last(), Some(&(2031, 2050)));
        // a year with no value is left out of the mean, not counted as nought
        let v = [1.0, 2.0, f64::NAN, 4.0, 5.0];
        assert_eq!(span_mean(&v, 1750, 1751, 1753), 3.0);
        assert_eq!(span_mean(&v, 1750, 1750, 1754), 3.0);
        assert!(span_mean(&v, 1750, 1752, 1752).is_nan());
    }

    #[test]
    fn least_squares_finds_a_line_it_was_given() {
        // y = 0.5 + 2 x0 - 3 x1, exactly
        let x: Vec<Vec<f64>> = (0..12).map(|k| vec![k as f64, ((k * 7) % 5) as f64]).collect();
        let y: Vec<f64> = x.iter().map(|r| 0.5 + 2.0 * r[0] - 3.0 * r[1]).collect();
        let fit = least_squares(&x, &y).unwrap();
        for (got, want) in fit.iter().zip([0.5, 2.0, -3.0]) {
            assert!((got - want).abs() < 1e-9, "{fit:?}");
        }
        // a column that never changes cannot be told from the constant
        let flat: Vec<Vec<f64>> = (0..12).map(|k| vec![k as f64, 1.0]).collect();
        assert!(least_squares(&flat, &y).is_none());
        assert!(least_squares(&x[..2], &y[..2]).is_none(), "fewer rows than coefficients");
    }
}
