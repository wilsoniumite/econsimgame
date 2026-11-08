//! The observer: turns smooth series into announced events. Python: `observer.py`.
//!
//! The simulation never reads anything written here. Monthly series go into a
//! registry; once a year the detectors look for
//!   - crossings: a new way of doing something overtakes the old one,
//!   - milestones: doublings, orders of magnitude, cities passing round numbers,
//!   - steepness: a year whose growth is far outside the trailing trend, labelled
//!     by how price and quantity moved together (demand boom, glut, shortage, slump),
//!   - downturns on two clocks: a live alarm when output falls, and a historian's
//!     verdict once the recovery is complete and the episode can be dated.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::Serialize;

use crate::content::Content;
use crate::params::Params;
use crate::state::{Event, State};
use crate::util::{fmt_pct, fmt_pct_signed, fmt_thousands, fmt_thousands_f0, pairwise_sum};

/// (name, kind, members): kind is "method", "service" or "heat".
pub const SHARE_GROUPS: &[(&str, &str, &[&str])] = &[
    ("iron", "method", &["bloomery", "charcoal_furnace", "coke_furnace"]),
    ("cloth", "method", &["hand_loom", "textile_mill"]),
    ("steel", "method", &["crucible_steel", "bessemer"]),
    ("rails", "method", &["forge_rails", "rolling_mill", "steel_rolling"]),
    ("machinery", "method", &["millwright", "engine_works"]),
    ("warmth", "service", &["timber", "charcoal", "coal"]),
    ("heat", "heat", &["timber", "charcoal", "coal"]),
    ("clothing", "service", &["wool", "cloth"]),
    ("shelter", "service", &["timber", "stone", "bricks"]),
    ("implements", "service", &["iron", "steel", "tools"]),
];

/// (group, member, title, text); `{share}` in the text is the share as a whole percentage.
pub const CROSSINGS: &[(&str, &str, &str, &str)] = &[
    ("iron", "coke_furnace", "Coke iron outweighs charcoal iron",
     "Coke furnaces now make {share} of the country's iron. Charcoal is dearer every year as the woods retreat; coal is not."),
    ("cloth", "textile_mill", "Mills now make most of the cloth",
     "Powered mills turn out {share} of all cloth. Handloom weavers are paid less each year for the same work."),
    ("clothing", "cloth", "Bought cloth overtakes homespun",
     "Households now get {share} of their clothing from bought cloth rather than wool spun at home."),
    ("steel", "bessemer", "Converter steel overtakes crucible steel",
     "Air-blown converters make {share} of the steel. What cost a week per pot now takes twenty minutes per charge."),
    ("rails", "rolling_mill", "Rolling mills overtake the rail forges",
     "Rolled rails are {share} of output, and the forges that hammered them out by hand are closing."),
    ("rails", "steel_rolling", "Most new rails are steel",
     "Steel rails are {share} of output. They cost a little more and last far longer."),
    ("machinery", "engine_works", "Engine works outbuild the millwrights",
     "Engine works now build {share} of all machinery. Millwrights go back to mending."),
    ("warmth", "coal", "Coal now heats more homes than wood",
     "Coal supplies {share} of household warmth. A coal fire is cheaper than a wood fire almost everywhere the roads reach."),
    ("heat", "coal", "Coal fires most of industry",
     "{share} of industrial heat now comes from coal."),
    ("shelter", "bricks", "Brick becomes the building material",
     "Bricks now provide {share} of new building and repair."),
];

pub const MILESTONE_CITY: [i64; 6] = [10_000, 25_000, 50_000, 100_000, 200_000, 400_000];

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct SeriesMeta {
    pub label: String,
    pub group: String,
    pub unit: String,
}

/// The series registry (`st["series"]` and `st["series_meta"]`): names in the order added.
#[derive(Clone, Debug, Default)]
pub struct Series {
    pub names: Vec<String>,
    pub meta: Vec<SeriesMeta>,
    pub data: Vec<Vec<f64>>,
    pub index: HashMap<String, usize>,
}

impl Series {
    fn add(&mut self, name: &str, label: &str, group: &str, unit: &str, n_months: usize) {
        self.index.insert(name.to_string(), self.names.len());
        self.names.push(name.to_string());
        self.meta.push(SeriesMeta { label: label.to_string(), group: group.to_string(), unit: unit.to_string() });
        self.data.push(vec![f64::NAN; n_months]);
    }

    pub fn get(&self, name: &str) -> &[f64] {
        &self.data[self.index[name]]
    }

    fn set(&mut self, name: &str, t: usize, v: f64) {
        let k = self.index[name];
        self.data[k][t] = v;
    }
}

/// The detectors' memory (`st["obs"]`).
#[derive(Clone, Debug, Default)]
pub struct ObserverState {
    /// (group, member) -> "above" or "below"
    pub cross_state: BTreeMap<(String, String), String>,
    /// Python keeps tuples and strings in one set; here each kind has a prefix.
    pub milestones: BTreeSet<String>,
    pub last_alarm: BTreeMap<String, i64>,
    pub peak: Option<(i64, f64)>,
    pub in_slump: bool,
    pub slump_start: Option<(i64, f64)>,
    pub trough: Option<(i64, f64)>,
    pub first_output: BTreeSet<usize>,
    /// `obs["annual"]["base"]`: the first year and value of each doubling series.
    pub base: BTreeMap<String, (i64, f64)>,
}

fn label_of(C: &Content, kind: &str, mem: &str) -> String {
    if kind == "method" {
        return C.method_label[crate::content::method_index(C, mem)].clone();
    }
    C.good_label[C.gi(mem)].clone()
}

pub fn init(st: &mut State, C: &Content, _P: &Params, n_months: usize) {
    let mut reg = Series::default();
    let mut add = |name: &str, label: &str, group: &str, unit: &str| reg.add(name, label, group, unit, n_months);
    add("pop", "Population", "people", "people");
    add("urban_share", "Living in towns", "people", "share");
    add("ag_share", "Working the land", "people", "share");
    add("ind_share", "Working in industry", "people", "share");
    add("gdp_nom", "Output (money)", "growth", "money/month");
    add("gdp_real", "Real output", "growth", "1750 money/month");
    add("gdp_pc", "Real output per head", "growth", "1750 money");
    add("cpi", "Cost of living", "prices", "1750 = 1");
    add("wage", "Average wage", "people", "money/month");
    add("real_wage", "Real wage", "people", "1750 = 1");
    add("rent_share", "Land rent share of output", "distribution", "share");
    add("invest_share", "Investment share of output", "growth", "share");
    add("lord_share_spend", "Landlords' share of spending", "distribution", "share");
    add("money", "Money stock", "money", "money");
    add("rail_km", "Railway", "transport", "km");
    add("road_km", "Roads", "transport", "km");
    add("towns", "Towns", "people", "count");
    add("firms", "Firms", "growth", "count");
    for (g, name) in C.good_name.iter().enumerate() {
        add(&format!("price_{name}"), &C.good_label[g], "prices", "money/unit");
        add(&format!("out_{name}"), &C.good_label[g], "output", "units/month");
    }
    for (m, name) in C.method_name.iter().enumerate() {
        add(&format!("mout_{name}"), &C.method_label[m], "methods", "units/month");
    }
    for &(gname, kind, members) in SHARE_GROUPS {
        for &mem in members {
            add(&format!("share_{gname}_{mem}"), &label_of(C, kind, mem), &format!("share_{gname}"), "share");
        }
    }
    st.series = reg;
    st.obs = ObserverState::default();
}

pub fn national_prices(st: &State) -> Vec<f64> {
    let T = st.T_max;
    let G = st.price.cols;
    let w: Vec<f64> = (0..T).map(|t| if st.town_active[t] { st.pop[t] } else { 0.0 }).collect();
    let wsum = pairwise_sum(&w).max(1.0);
    let mut out = vec![0.0; G];
    for t in 0..T {
        for g in 0..G {
            out[g] += st.price[(t, g)] * w[t];
        }
    }
    out.iter().map(|v| v / wsum).collect()
}

pub fn record_month(st: &mut State, C: &Content, _P: &Params, t: usize) {
    let T = st.T_max;
    let ms = &st.month_stats;
    let pop: Vec<f64> = (0..T).map(|i| if st.town_active[i] { st.pop[i] } else { 0.0 }).collect();
    let pop_sum = pairwise_sum(&pop);
    let P_nat = national_prices(st);
    let mut s = std::mem::take(&mut st.series);
    s.set("pop", t, pop_sum);
    let urban: Vec<f64> = (0..T).map(|i| if st.town_active[i] { st.urban_pop[i] } else { 0.0 }).collect();
    s.set("urban_share", t, pairwise_sum(&urban) / pop_sum.max(1.0));
    let emp = pairwise_sum(&ms.employment).max(1e-9);
    let ag = pairwise_sum(&ms.ex_labor_by_sector[..3]);
    s.set("ag_share", t, ag / emp);
    s.set("ind_share", t, pairwise_sum(&ms.firm_labor_by_method) / emp);
    let gdp = ms.hh_spend + ms.lord_spend + ms.urban_rent + st.m_invest;
    s.set("gdp_nom", t, gdp);
    let basket = &st.cpi_basket;
    let G = C.G;
    let cpi = crate::util::pairwise_sum_by(G, |g| basket[g] * P_nat[g])
        / crate::util::pairwise_sum_by(G, |g| basket[g] * st.cpi_base_prices[g]).max(1e-9);
    s.set("cpi", t, cpi);
    let gdp_real = gdp / cpi.max(1e-9);
    s.set("gdp_real", t, gdp_real);
    s.set("gdp_pc", t, gdp_real / pop_sum.max(1.0));
    let wage = crate::util::pairwise_sum_by(T, |i| st.wage[i] * pop[i]) / pop_sum.max(1.0);
    s.set("wage", t, wage);
    s.set("real_wage", t, wage / cpi.max(1e-9) / st.wage_base);
    s.set("rent_share", t, st.m_rent / gdp.max(1e-9));
    s.set("invest_share", t, st.m_invest / gdp.max(1e-9));
    s.set("lord_share_spend", t, ms.lord_spend / (ms.hh_spend + ms.lord_spend).max(1e-9));
    s.set("money", t, st.money_now);
    s.set("rail_km", t, st.rail.iter().map(|&r| r as i64).sum::<i64>() as f64 * 2.0);
    s.set("road_km", t, st.road.iter().filter(|&&r| r > 1).count() as f64 * 2.0);
    s.set("towns", t, st.town_active.iter().filter(|&&a| a).count() as f64);
    s.set("firms", t, st.f_alive.iter().filter(|&&a| a).count() as f64);
    for (g, name) in C.good_name.iter().enumerate() {
        s.set(&format!("price_{name}"), t, P_nat[g]);
        s.set(&format!("out_{name}"), t, ms.prod[g]);
    }
    for (m, name) in C.method_name.iter().enumerate() {
        s.set(&format!("mout_{name}"), t, ms.method_out[m]);
    }
    // shares
    let mut hh = vec![0.0; G];
    for i in 0..T {
        for g in 0..G {
            hh[g] += ms.hh_got[(i, g)];
        }
    }
    let mut lord = vec![0.0; G];
    for i in 0..T {
        for g in 0..G {
            lord[g] += ms.lord_got[(i, g)];
        }
    }
    for g in 0..G {
        hh[g] += lord[g];
    }
    for &(gname, kind, members) in SHARE_GROUPS {
        let v: Vec<f64> = match kind {
            "method" => members.iter().map(|&x| ms.method_out[crate::content::method_index(C, x)]).collect(),
            "service" => {
                let sidx = C.service_name.iter().position(|n| n == gname).unwrap();
                members.iter().map(|&x| hh[C.gi(x)] * C.props[(C.gi(x), sidx)]).collect()
            }
            _ => members
                .iter()
                .map(|&x| {
                    let k = C.fuel_goods.iter().position(|&g| g == C.gi(x)).unwrap();
                    ms.heat_use[k]
                })
                .collect(),
        };
        let tot = pairwise_sum(&v);
        for (mem, val) in members.iter().zip(&v) {
            s.set(&format!("share_{gname}_{mem}"), t, if tot > 1e-9 { val / tot } else { f64::NAN });
        }
    }
    st.series = s;
}

/// Mean of a monthly series over calendar year y.
pub fn annual(series: &[f64], year0: i64, y: i64) -> f64 {
    let i = (y - year0) * 12;
    if i < 0 || i as usize >= series.len() {
        return f64::NAN;
    }
    let i = i as usize;
    let v: Vec<f64> = series[i..(i + 12).min(series.len())].iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() { f64::NAN } else { pairwise_sum(&v) / v.len() as f64 }
}

/// numpy's `np.round(x, n)` (as `round()` does on a numpy float): scale, round half to even, unscale.
fn np_round(x: f64, n: i32) -> f64 {
    let s = 10f64.powi(n);
    (x * s).round_ties_even() / s
}

pub fn yearly(st: &mut State, C: &Content, P: &Params, year: i64, events: &mut Vec<Event>) {
    let y0 = C.start_year;
    let k = year - y0;
    let s = std::mem::take(&mut st.series);
    let mut ob = std::mem::take(&mut st.obs);

    // ---- crossings, with hysteresis and a three-month confirmation
    for &(group, mem, title, text) in CROSSINGS {
        let name = format!("share_{group}_{mem}");
        let ser = s.get(&name);
        let i = (k * 12) as usize;
        let recent: Vec<f64> = if i + 9 < ser.len() {
            ser[i + 9..(i + 12).min(ser.len())].iter().copied().filter(|x| x.is_finite()).collect()
        } else {
            Vec::new()
        };
        if recent.is_empty() {
            continue;
        }
        let v = pairwise_sum(&recent) / recent.len() as f64;
        let key = (group.to_string(), mem.to_string());
        let mkey = format!("cross:{group}:{mem}");
        let Some(state) = ob.cross_state.get(&key).cloned() else {
            ob.cross_state.insert(key, if v > P.crossing_hi { "above" } else { "below" }.to_string());
            if v > P.crossing_hi {
                ob.milestones.insert(mkey);
            }
            continue;
        };
        if state == "below" && v > P.crossing_hi {
            ob.cross_state.insert(key, "above".to_string());
            let first = !ob.milestones.contains(&mkey);
            ob.milestones.insert(mkey);
            let mut ev = Event::new(
                year as f64 + 0.8,
                "crossing",
                if first { 0.85 } else { 0.4 },
                if first { title.to_string() } else { format!("{title} again") },
                text.replace("{share}", &fmt_pct(v, 0)),
            );
            ev.series = Some(name);
            events.push(ev);
        } else if state == "above" && v < P.crossing_lo {
            ob.cross_state.insert(key, "below".to_string());
        }
    }

    // structural crossings
    let ag = annual(s.get("ag_share"), y0, year);
    let ind = annual(s.get("ind_share"), y0, year);
    let urb = annual(s.get("urban_share"), y0, year);
    if ind.is_finite() && ind > ag && !ob.milestones.contains("ind>ag") && k > 0 {
        ob.milestones.insert("ind>ag".to_string());
        let mut ev = Event::new(
            year as f64 + 0.8,
            "crossing",
            0.95,
            "More hands in workshops and mills than on the land".to_string(),
            format!("Industry employs {} of workers; farms, pastures and woods {}.", fmt_pct(ind, 0), fmt_pct(ag, 0)),
        );
        ev.series = Some("ind_share".to_string());
        events.push(ev);
    }
    for (thr, label) in [
        (0.25, "A quarter of the people now live in towns"),
        (0.4, "Two in five people now live in towns"),
        (0.5, "Most people now live in towns"),
    ] {
        let key = format!("urban{thr}");
        if urb.is_finite() && urb > thr && !ob.milestones.contains(&key) {
            ob.milestones.insert(key);
            if k == 0 {
                continue;
            }
            let mut ev = Event::new(
                year as f64 + 0.8,
                "milestone",
                0.7,
                label.to_string(),
                format!("{} of the population lives in towns.", fmt_pct(urb, 0)),
            );
            ev.series = Some("urban_share".to_string());
            events.push(ev);
        }
    }

    // ---- doublings and orders of magnitude
    for (name, label, kind) in [
        ("gdp_real", "Real output", 2i64),
        ("pop", "Population", 2),
        ("gdp_pc", "Output per head", 2),
        ("real_wage", "The real wage", 2),
        ("out_iron", "Iron output", 10),
        ("out_coal", "Coal output", 10),
        ("out_cloth", "Cloth output", 10),
    ] {
        let v = annual(s.get(name), y0, year);
        if !v.is_finite() || v <= 0.0 {
            continue;
        }
        let Some(&(when, v0)) = ob.base.get(name) else {
            ob.base.insert(name.to_string(), (year, v));
            continue;
        };
        let ratio = v / v0;
        let lvl = if ratio >= kind as f64 { (ratio.ln() / (kind as f64).ln() + 1e-9).floor() as i64 } else { 0 };
        let key = format!("{name}:{lvl}");
        if lvl >= 1 && !ob.milestones.contains(&key) {
            ob.milestones.insert(key);
            let mult = kind.pow(lvl as u32);
            let verb = if mult == 2 { "doubles".to_string() } else { format!("is up {mult}x") };
            let weight = if matches!(name, "gdp_real" | "gdp_pc" | "real_wage") { 0.75 } else { 0.5 };
            let mut ev = Event::new(
                year as f64 + 0.7,
                "milestone",
                weight,
                format!("{label} {verb} since {when}"),
                format!("{label} is now {ratio:.1} times its {when} level."),
            );
            ev.series = Some(name.to_string());
            events.push(ev);
        }
    }

    // ---- the railway network passing round numbers
    let km = annual(s.get("rail_km"), y0, year);
    for thr in [250i64, 500, 1000, 2000] {
        let key = format!("rail:{thr}");
        if km.is_finite() && km >= thr as f64 && !ob.milestones.contains(&key) {
            ob.milestones.insert(key);
            let mut ev = Event::new(
                year as f64 + 0.8,
                "milestone",
                0.65,
                format!("The railway network passes {} km", fmt_thousands(thr)),
                format!("There are now {} km of railway in the country.", fmt_thousands_f0(km)),
            );
            ev.series = Some("rail_km".to_string());
            events.push(ev);
        }
    }

    // ---- cities passing round numbers
    for i in 0..st.T_max {
        if !st.town_active[i] {
            continue;
        }
        let u = st.urban_pop[i];
        for (idx, &thr) in MILESTONE_CITY.iter().enumerate() {
            let key = format!("city:{thr}");
            if u >= thr as f64 && !ob.milestones.contains(&key) {
                ob.milestones.insert(key);
                if k == 0 {
                    continue;
                }
                let mut ev = Event::new(
                    year as f64 + 0.75,
                    "milestone",
                    0.55 + 0.05 * idx as f64,
                    format!("{} passes {} people", st.town_name[i], fmt_thousands(thr)),
                    format!("{} is the first town in the country with more than {} inhabitants.", st.town_name[i], fmt_thousands(thr)),
                );
                ev.town = Some(i);
                ev.cell = Some(st.town_node[i]);
                events.push(ev);
            }
        }
    }

    // ---- steepness: this year's growth against the trailing decade
    let mut steep: Vec<(String, String, Event)> = Vec::new();
    if k >= 6 {
        for gname in ["grain", "coal", "iron", "cloth", "machinery", "rails", "steel", "timber", "charcoal"] {
            let g = C.gi(gname);
            let years: Vec<i64> = (y0.max(year - 11)..=year).collect();
            let q: Vec<f64> = years.iter().map(|&yy| annual(s.get(&format!("out_{gname}")), y0, yy)).collect();
            let p: Vec<f64> = years.iter().map(|&yy| annual(s.get(&format!("price_{gname}")), y0, yy)).collect();
            if !q.iter().all(|x| x.is_finite()) || q.iter().cloned().fold(f64::INFINITY, f64::min) <= 1e-6 {
                continue;
            }
            let dq: Vec<f64> = q.windows(2).map(|w| w[1].ln() - w[0].ln()).collect();
            let dp: Vec<f64> = p.windows(2).map(|w| w[1].ln() - w[0].ln()).collect();
            let (hist, now) = (&dq[..dq.len() - 1], dq[dq.len() - 1]);
            let mean = pairwise_sum(hist) / hist.len() as f64;
            let dev: Vec<f64> = hist.iter().map(|x| (x - mean) * (x - mean)).collect();
            let std = (pairwise_sum(&dev) / hist.len() as f64).sqrt();
            let z = (now - mean) / std.max(0.04);
            let last = ob.last_alarm.get(gname).copied().unwrap_or(-99);
            if year - last < 4 || z.abs() < 2.5 {
                continue;
            }
            if now.abs() < 0.12 {
                continue;
            }
            let share = annual(s.get(&format!("out_{gname}")), y0, year) * annual(s.get(&format!("price_{gname}")), y0, year)
                / annual(s.get("gdp_nom"), y0, year).max(1e-9);
            let (up_q, up_p) = (now > 0.0, dp[dp.len() - 1] > 0.0);
            let label = &C.good_label[g];
            let (title, why) = match (up_q, up_p) {
                (true, true) => (format!("{label} boom"), "demand is outrunning supply"),
                (true, false) => (format!("{label} glut"), "new capacity is flooding the market"),
                (false, true) => (format!("{label} shortage"), "supply has failed while buyers still want it"),
                (false, false) => (format!("{label} slump"), "buyers have stopped buying"),
            };
            ob.last_alarm.insert(gname.to_string(), year);
            let text = format!(
                "{label} output {} {} this year against a trend of {} a year, while prices {} {}: {why}.",
                if up_q { "rose" } else { "fell" },
                fmt_pct(now.exp_m1().abs(), 0),
                fmt_pct_signed(mean.exp_m1(), 0),
                if up_p { "rose" } else { "fell" },
                fmt_pct(dp[dp.len() - 1].exp_m1().abs(), 0),
            );
            let weight = (0.35 + 2.0 * share + 0.05 * z.abs()).clamp(0.35, 0.8);
            let kind = title.split_whitespace().last().unwrap().to_string();
            let mut ev = Event::new(year as f64 + 0.85, "steep", weight, title, text);
            ev.series = Some(format!("out_{gname}"));
            steep.push((kind, label.clone(), ev));
        }
    }
    // group by kind in order of first appearance, as a Python dict does
    let mut by_kind: Vec<(String, Vec<(String, Event)>)> = Vec::new();
    for (kind, label, ev) in steep {
        match by_kind.iter_mut().find(|(k, _)| *k == kind) {
            Some((_, items)) => items.push((label, ev)),
            None => by_kind.push((kind, vec![(label, ev)])),
        }
    }
    for (kind, items) in by_kind {
        if items.len() >= 3 {
            let labels: Vec<&str> = items.iter().map(|(l, _)| l.as_str()).collect();
            let listing = format!("{} and {}", labels[..labels.len() - 1].join(", "), labels[labels.len() - 1]);
            let (what, why) = match kind.as_str() {
                "shortage" => ("Shortages", "Output fell while buyers kept bidding."),
                "glut" => ("Gluts", "New capacity is flooding markets and prices are falling."),
                "boom" => ("Booms", "Demand is running ahead of supply and prices are rising."),
                _ => ("Slumps", "Buyers have pulled back; output and prices are both falling."),
            };
            let mut ev = Event::new(
                year as f64 + 0.85,
                "steep",
                0.7,
                format!("{what} in {} industries", items.len()),
                format!("{listing} all moved far outside their trend this year. {why}"),
            );
            ev.series = items[0].1.series.clone();
            events.push(ev);
        } else {
            events.extend(items.into_iter().map(|(_, ev)| ev));
        }
    }

    // ---- downturns, on two clocks
    let gdp = annual(s.get("gdp_real"), y0, year);
    let prev = if k >= 1 { annual(s.get("gdp_real"), y0, year - 1) } else { f64::NAN };
    if gdp.is_finite() && prev.is_finite() {
        if ob.peak.is_none() || (!ob.in_slump && gdp > ob.peak.unwrap().1) {
            ob.peak = Some((year, gdp));
        }
        let fall = gdp / prev - 1.0;
        if !ob.in_slump && fall < -0.02 {
            ob.in_slump = true;
            ob.slump_start = ob.peak;
            ob.trough = Some((year, gdp));
            let x = 0.45 + 4.0 * -fall;
            let weight = if x < 0.8 { np_round(x, 3) } else { 0.8 };
            let mut ev = Event::new(
                year as f64 + 0.9,
                "alarm",
                weight,
                format!("Output falls {}", fmt_pct(-fall, 1)),
                format!("Real output fell {} this year. Nobody yet knows how long it will last.", fmt_pct(-fall, 1)),
            );
            ev.series = Some("gdp_real".to_string());
            events.push(ev);
        } else if ob.in_slump {
            if gdp < ob.trough.unwrap().1 {
                ob.trough = Some((year, gdp));
            }
            if gdp > ob.slump_start.unwrap().1 {
                let ((py, pv), (ty, tv)) = (ob.slump_start.unwrap(), ob.trough.unwrap());
                let depth = 1.0 - tv / pv;
                let big = depth > 0.08;
                let mut ev = Event::new(
                    year as f64 + 0.95,
                    "verdict",
                    if depth > 0.04 { 0.75 } else { 0.5 },
                    format!("The {} of {py}\u{2013}{ty}", if big { "depression" } else { "slump" }),
                    format!(
                        "Output fell {} from its {py} peak to the {ty} trough and took until {year} to recover. Historians will call this a {}.",
                        fmt_pct(depth, 1),
                        if big { "depression" } else { "recession" }
                    ),
                );
                ev.clock = "verdict".to_string();
                ev.span = Some([py as f64 + 0.5, ty as f64 + 0.5]);
                ev.series = Some("gdp_real".to_string());
                events.push(ev);
                ob.in_slump = false;
                ob.peak = Some((year, gdp));
            }
        }
    }

    // ---- first production by newly discovered methods
    for &fid in &st.new_firms_year {
        let m = st.f_method[fid];
        if C.tech_known[m] || ob.first_output.contains(&m) {
            continue;
        }
        ob.first_output.insert(m);
        let i = st.f_town[fid];
        let mut ev = Event::new(
            year as f64 + 0.9,
            "first",
            0.5,
            format!("{} founded", st.f_name[fid]),
            format!("The first {} business in the country sets up near {}.", C.method_label[m].to_lowercase(), st.town_name[i]),
        );
        ev.town = Some(i);
        ev.cell = Some(st.f_cell[fid]);
        ev.firm = Some(fid);
        events.push(ev);
    }
    st.series = s;
    st.obs = ob;
}
