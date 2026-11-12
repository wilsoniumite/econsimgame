//! Prints, from the books of account, the building cycle (how much of each town's year's room its trades take, timber's price, what a wage buys), the famine of tools (who buys, who goes short, what the smithies do) and which goods swing most, as means over seeds; written for docs/probes/tr-cycles.md.
//!
//!     cargo run --release -p isostate-core --example trace_cycles -- [--seeds 1-8] [--families 6]
//!         [--set key=value]... [--jobs 8] [--detail SEED] [--from 1780] [--json FILE] [--compare FILE]
//!
//! Every run is under the game's rules with the books on, played to 1900 a year
//! at a time; after each year the example reads what each trade in each town
//! ordered that year (`State::works_added`) against its year's room
//! (`works::year_budget`), which the books do not keep. Everything else comes
//! from the books and the observer's series once the run is over.
//!
//! The numbers, each a mean over seeds with the least and greatest seed:
//!
//! - swings are the standard deviation of the yearly change in the log of a
//!   yearly mean, over 1800 to 1900; "amp" is the standard deviation of the log
//!   price about its own centred nine-year mean, and "period" the period (2 to 12
//!   years) at which that deviation has most power. "Dearest" is the dearest
//!   year over the median year, from `--from` (1780) on.
//! - a trade's "room taken" in a year is what it ordered over its room; a trade
//!   is counted in a town only where it has capacity or has just ordered some.
//!   "Whole" years are those it took 0.9 of its room or more, "bare" years 0.1
//!   or less.
//! - "ordered" timber or tools is capacity ordered times what a unit of the
//!   method's plant takes of the good (`m_cap`): what the order will ask the
//!   market for over the following months.
//!
//! `--detail SEED` adds that seed year by year. `--json FILE` writes every
//! number of every seed; `--compare FILE` sets the means beside another such
//! file's, with a star where the two ranges across seeds do not overlap.
#![allow(non_snake_case)]

use std::collections::BTreeMap;

use isostate_core::accounts::{GOODS_FIELDS, KG, KM, METHOD_FIELDS};
use isostate_core::sweep::{parse_seeds, parse_value, set_field};
use isostate_core::*;

/// What the books do not keep: per year, `[town * M + method]`.
struct Orders {
    added: Vec<f64>,
    budget: Vec<f64>,
    have: Vec<f64>,
}

struct Run {
    seed: u64,
    r: RunResult,
    orders: Vec<Orders>,
    /// `[year][traced method]`: the review's view of a trade, taken after December.
    review: Vec<Vec<Review>>,
}

/// The methods whose works are followed into the review.
const TRACED: [&str; 4] = ["textile_mill", "brickworks", "smithy", "engine_works"];

/// How a trade's works stood as the yearly review was about to judge them, as
/// shares of the trade's built capacity: free to grow (a return over the hurdle,
/// running over 0.65, not building), still building, a return under the hurdle,
/// and slack (a return over the hurdle but running at 0.65 or less). Then the
/// mean return on plant as the review reckons it (the year's profit over the
/// capacity at this year's prices of the plant), and the price of a unit of
/// plant over its price at the content's base prices.
#[derive(Clone, Copy, Default)]
struct Review {
    grow: f64,
    building: f64,
    low: f64,
    slack: f64,
    roi: f64,
    plant_price: f64,
}

fn look(st: &State, C: &Content, P: &Params, m: usize) -> Review {
    let mut r = Review::default();
    let (mut cap, mut roi_w, mut pp_w) = (0.0, 0.0, 0.0);
    let base: f64 = (0..C.G).map(|g| C.m_cap[(m, g)] * C.base_price[g]).sum();
    for f in 0..st.f_alive.len() {
        if !st.f_alive[f] || st.f_method[f] != m || st.f_K[f] <= 0.0 {
            continue;
        }
        let (i, K) = (st.f_town[f], st.f_K[f]);
        // the review's own prices: the year's mean (`sim::yearly`)
        let unit_cap: f64 = (0..C.G).map(|g| C.m_cap[(m, g)] * st.price_acc[(i, g)] / 12.0).sum();
        let roi = st.f_profit_acc[f] / (K * unit_cap).max(1e-9);
        let building = works::building(P, K, st.f_order_K[f]);
        cap += K;
        roi_w += K * roi;
        pp_w += K * unit_cap / base.max(1e-12);
        if building {
            r.building += K;
        } else if roi <= P.hurdle {
            r.low += K;
        } else if st.f_u[f] <= 0.65 {
            r.slack += K;
        } else {
            r.grow += K;
        }
    }
    if cap > 0.0 {
        r.grow /= cap;
        r.building /= cap;
        r.low /= cap;
        r.slack /= cap;
        r.roi = roi_w / cap;
        r.plant_price = pp_w / cap;
    }
    r
}

fn play(P: &Params, seed: u64) -> Run {
    let mut P = P.clone();
    P.seed = seed;
    let mut sim = sim::start(&P, Rules::Game.content());
    let mut orders = Vec::new();
    let mut review = Vec::new();
    while sim.year <= sim.end_year {
        for month in 0..12 {
            sim::run_month(&mut sim, month).expect("money is conserved every month");
        }
        {
            let (st, C, P) = (&sim.st, &sim.content, &sim.params);
            review.push(TRACED.iter().map(|name| look(st, C, P, C.method_name.iter().position(|x| x == name).expect("a method"))).collect::<Vec<_>>());
        }
        sim::end_year(&mut sim);
        let (st, C, P) = (&sim.st, &sim.content, &sim.params);
        let (T, M) = (st.T_max, C.M);
        let mut o = Orders { added: vec![0.0; T * M], budget: vec![0.0; T * M], have: vec![0.0; T * M] };
        for f in 0..st.f_alive.len() {
            if st.f_alive[f] {
                o.have[st.f_town[f] * M + st.f_method[f]] += st.f_K[f] + st.f_order_K[f];
            }
        }
        for t in 0..T {
            if !st.town_active[t] {
                continue;
            }
            for m in 0..M {
                if st.works_added.rows > 0 {
                    o.added[t * M + m] = st.works_added[(t, m)];
                }
                o.budget[t * M + m] = works::year_budget(st, C, P, t, m);
            }
        }
        orders.push(o);
    }
    Run { seed, r: sim::finish(sim), orders, review }
}

// ---------------------------------------------------------------- small statistics

fn mean(v: &[f64]) -> f64 {
    let v: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 }
}

fn sd(v: &[f64]) -> f64 {
    let m = mean(v);
    let v: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if v.len() < 2 { f64::NAN } else { (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / v.len() as f64).sqrt() }
}

fn corr(a: &[f64], b: &[f64]) -> f64 {
    let pairs: Vec<(f64, f64)> = a.iter().zip(b).filter(|(x, y)| x.is_finite() && y.is_finite()).map(|(&x, &y)| (x, y)).collect();
    if pairs.len() < 3 {
        return f64::NAN;
    }
    let (ma, mb) = (mean(&pairs.iter().map(|p| p.0).collect::<Vec<_>>()), mean(&pairs.iter().map(|p| p.1).collect::<Vec<_>>()));
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for (x, y) in pairs {
        sab += (x - ma) * (y - mb);
        saa += (x - ma) * (x - ma);
        sbb += (y - mb) * (y - mb);
    }
    if saa > 0.0 && sbb > 0.0 { sab / (saa * sbb).sqrt() } else { f64::NAN }
}

fn median(v: &[f64]) -> f64 {
    let mut s: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if s.is_empty() {
        return f64::NAN;
    }
    s.sort_by(f64::total_cmp);
    let n = s.len();
    if n % 2 == 1 { s[n / 2] } else { 0.5 * (s[n / 2 - 1] + s[n / 2]) }
}

/// Yearly changes of the log.
fn dlog(v: &[f64]) -> Vec<f64> {
    v.windows(2).map(|w| (w[1] / w[0]).ln()).map(|d| if d.is_finite() { d } else { f64::NAN }).collect()
}

/// The log of each year about its centred nine-year mean of logs (NaN at the ends).
fn dev(v: &[f64]) -> Vec<f64> {
    let l: Vec<f64> = v.iter().map(|x| x.ln()).collect();
    (0..l.len()).map(|k| if k >= 4 && k + 4 < l.len() { l[k] - mean(&l[k - 4..=k + 4]) } else { f64::NAN }).collect()
}

/// The period, 2 to 12 years by quarters, at which a deviation has most power.
fn period(d: &[f64]) -> f64 {
    let x: Vec<(f64, f64)> = d.iter().enumerate().filter(|(_, v)| v.is_finite()).map(|(k, &v)| (k as f64, v)).collect();
    if x.len() < 20 {
        return f64::NAN;
    }
    let mut best = (f64::NAN, -1.0);
    let mut p = 2.0;
    while p <= 12.0 + 1e-9 {
        let w = std::f64::consts::TAU / p;
        let (c, s) = x.iter().fold((0.0, 0.0), |(c, s), &(k, v)| (c + v * (w * k).cos(), s + v * (w * k).sin()));
        if c * c + s * s > best.1 {
            best = (p, c * c + s * s);
        }
        p += 0.25;
    }
    best.0
}

/// Lag-one autocorrelation.
fn ac1(d: &[f64]) -> f64 {
    if d.len() < 3 { f64::NAN } else { corr(&d[..d.len() - 1], &d[1..]) }
}

// ---------------------------------------------------------------- what each run gives

/// Every number of one run by name, and the years for `--detail`.
struct Out {
    nums: BTreeMap<String, f64>,
    detail: Vec<String>,
}

fn col(fields: &[&str], name: &str) -> usize {
    fields.iter().position(|&n| n == name).unwrap_or_else(|| panic!("no column {name}"))
}

fn reduce(run: &Run, from: i64, want_detail: bool) -> Out {
    let (st, C, rec) = (&run.r.state, &run.r.content, &run.r.rec);
    let y0 = C.start_year;
    let n = rec.n_years;
    let (T, G, M) = (st.T_max, C.G, C.M);
    let ann = facts::annual_means(&run.r);
    let series = |name: &str| -> Vec<f64> { st.series.index.get(name).map_or(vec![f64::NAN; n], |&k| ann[k].clone()) };
    let k1800 = (1800 - y0) as usize;
    let kfrom = (from - y0).max(0) as usize;
    let mut nums = BTreeMap::new();
    let mut put = |k: &str, v: f64| {
        nums.insert(k.to_string(), v);
    };
    let b = &st.acct;
    let gcol = |name: &str| col(&GOODS_FIELDS, name);
    let mcol = |name: &str| col(&METHOD_FIELDS, name);
    let buyers = gcol("households")..=gcol("projects");
    // island totals of a column of the goods' books, a year
    let island = |k: usize, g: usize, c: usize| -> f64 {
        let y = b.year(k).expect("a year's books");
        (0..T).map(|t| y.goods[(t * G + g) * KG + c] as f64).sum()
    };
    let bought = |k: usize, g: usize| -> f64 { buyers.clone().map(|c| island(k, g, c)).sum() };
    let method = |k: usize, m: usize, c: usize| -> f64 {
        let y = b.year(k).expect("a year's books");
        (0..T).map(|t| y.methods[(t * M + m) * KM + c] as f64).sum()
    };
    let mi = |name: &str| C.method_name.iter().position(|x| x == name).unwrap_or_else(|| panic!("no method {name}"));
    let (timber, tools, machinery) = (C.gi("timber"), C.gi("tools"), C.gi("machinery"));
    let (mill, smithy, engine, millwright) = (mi("textile_mill"), mi("smithy"), mi("engine_works"), mi("millwright"));

    // ---- (a) the building cycle
    let p_timber = series("price_timber");
    let rw = series("real_wage");
    let cpi = series("cpi");
    let late = |v: &[f64]| v[k1800..].to_vec();
    put("timber_swing", sd(&dlog(&late(&p_timber))));
    put("timber_amp", sd(&late(&dev(&p_timber))));
    put("timber_period", period(&late(&dev(&p_timber))));
    put("timber_ac1", ac1(&dlog(&late(&p_timber))));
    // a spike: a year over twice the median of the nine years about it
    let spikes: Vec<f64> = (k1800.max(4)..n.saturating_sub(4)).map(|k| p_timber[k] / median(&p_timber[k - 4..=k + 4])).collect();
    put("timber_spike_years", spikes.iter().filter(|&&r| r > 2.0).count() as f64);
    put("timber_highest_over_local_median", spikes.iter().copied().fold(f64::NAN, f64::max));
    put("real_wage_swing", sd(&dlog(&late(&rw))));
    put("real_wage_worst_year", dlog(&late(&rw)).iter().copied().fold(f64::NAN, f64::min).exp_m1());
    put("real_wage_amp", sd(&late(&dev(&rw))));
    put("cpi_swing", sd(&dlog(&late(&cpi))));
    // levels, as twenty-year means, so that a cure's cost can be read beside its gain
    let gdp_pc = series("gdp_pc");
    for (a, z) in [(1830i64, 1849i64), (1880, 1899)] {
        let (ka, kz) = (((a - y0).max(0) as usize).min(n - 1), ((z - y0).max(0) as usize).min(n - 1));
        put(&format!("level_real_wage_{a}_{z}"), mean(&rw[ka..=kz]));
        put(&format!("level_output_per_head_{a}_{z}"), mean(&gdp_pc[ka..=kz]));
    }
    put("level_works_1900", st.f_alive.iter().filter(|&&a| a).count() as f64);
    put("level_pop_1900", series("pop")[n - 1]);
    put("real_wage_vs_timber_dev_corr", corr(&late(&dev(&rw)), &late(&dev(&p_timber))));
    put("real_wage_vs_timber_change_corr", corr(&dlog(&late(&rw)), &dlog(&late(&p_timber))));
    // timber bought by works being built, as a share of all timber bought
    let tb_share: Vec<f64> = (0..n).map(|k| island(k, timber, gcol("building")) / bought(k, timber).max(1e-9)).collect();
    put("timber_building_share", mean(&tb_share[k1800..]));
    put("timber_building_share_max", tb_share[k1800..].iter().copied().fold(f64::NAN, f64::max));
    let tb_q: Vec<f64> = (0..n).map(|k| island(k, timber, gcol("building"))).collect();
    put("timber_price_vs_building_dev_corr", corr(&late(&dev(&p_timber)), &late(&dev(&tb_q))));
    // where timber comes from: the woods' own cutting, or woodland cleared at the year's end
    let t_made: Vec<f64> = (0..n).map(|k| island(k, timber, gcol("made"))).collect();
    let t_clear: Vec<f64> = (0..n).map(|k| island(k, timber, gcol("cleared"))).collect();
    put("timber_cleared_share", (k1800..n).map(|k| t_clear[k]).sum::<f64>() / (k1800..n).map(|k| t_made[k]).sum::<f64>().max(1e-9));
    put("timber_made_swing", sd(&dlog(&late(&t_made))));
    put("timber_made_vs_price_dev_corr", corr(&late(&dev(&t_made)), &late(&dev(&p_timber))));
    put("timber_made_vs_price_before_dev_corr", corr(&dev(&t_made)[k1800 + 1..], &dev(&p_timber)[k1800..n - 1]));
    let t_hh: Vec<f64> = (0..n).map(|k| island(k, timber, gcol("households")) + island(k, timber, gcol("landlords")) + island(k, timber, gcol("families"))).collect();
    put("timber_households_share", (k1800..n).map(|k| t_hh[k]).sum::<f64>() / (k1800..n).map(|k| bought(k, timber)).sum::<f64>().max(1e-9));
    put("timber_households_swing", sd(&dlog(&late(&t_hh))));
    let t_unmet: Vec<f64> = (0..n).map(|k| 1.0 - bought(k, timber) / island(k, timber, gcol("asked")).max(1e-9)).collect();
    put("timber_unmet", mean(&t_unmet[k1800..]));
    put("timber_unmet_worst", t_unmet[k1800..].iter().copied().fold(f64::NAN, f64::max));
    // timber ordered for plant, by method: capacity ordered times the plant's timber
    let ordered_by = |k: usize, m: usize, g: usize| -> f64 { (0..T).map(|t| run.orders[k].added[t * M + m]).sum::<f64>() * C.m_cap[(m, g)] };
    let ordered_all = |k: usize, g: usize| -> f64 { (0..M).map(|m| ordered_by(k, m, g)).sum() };
    let tot_timber: f64 = (k1800..n).map(|k| ordered_all(k, timber)).sum();
    for m in 0..M {
        if C.m_cap[(m, timber)] > 0.0 {
            put(&format!("timber_ordered_share_{}", C.method_name[m]), (k1800..n).map(|k| ordered_by(k, m, timber)).sum::<f64>() / tot_timber.max(1e-9));
        }
    }
    // a year's orders of timber for plant, over a month of the island's timber sales
    let months_of_sales: Vec<f64> = (0..n).map(|k| ordered_all(k, timber) / (bought(k, timber) / 12.0).max(1e-9)).collect();
    put("timber_ordered_months_of_sales", mean(&months_of_sales[k1800..]));
    put("timber_ordered_months_of_sales_max", months_of_sales[k1800..].iter().copied().fold(f64::NAN, f64::max));
    // the room: what each trade took of it, town by town
    let taken = |k: usize, t: usize, m: usize| -> Option<f64> {
        let o = &run.orders[k];
        let j = t * M + m;
        let have_before = if k > 0 { run.orders[k - 1].have[j] } else { 0.0 };
        if o.budget[j] > 0.0 && (have_before > 0.0 || o.added[j] > 0.0) { Some(o.added[j] / o.budget[j]) } else { None }
    };
    for (label, m) in [("mill", mill), ("smithy", smithy), ("engine", engine), ("millwright", millwright), ("hand_loom", mi("hand_loom")), ("brickworks", mi("brickworks"))] {
        let u: Vec<f64> = (k1800..n).flat_map(|k| (0..T).filter_map(move |t| taken(k, t, m))).collect();
        put(&format!("room_{label}_mean"), mean(&u));
        put(&format!("room_{label}_whole"), u.iter().filter(|&&x| x >= 0.9).count() as f64 / u.len().max(1) as f64);
        put(&format!("room_{label}_bare"), u.iter().filter(|&&x| x <= 0.1).count() as f64 / u.len().max(1) as f64);
    }
    // mills: years between whole years, and how far the towns move together
    let mut gaps = Vec::new();
    let mut by_town: Vec<Vec<f64>> = vec![Vec::new(); T];
    for t in 0..T {
        let mut last = None;
        for k in k1800..n {
            let u = taken(k, t, mill);
            by_town[t].push(u.unwrap_or(f64::NAN));
            if u.is_some_and(|x| x >= 0.9) {
                if let Some(l) = last {
                    gaps.push((k - l) as f64);
                }
                last = Some(k);
            }
        }
    }
    put("mill_years_between_whole", mean(&gaps));
    let mut pair = Vec::new();
    for a in 0..T {
        for c in a + 1..T {
            let both = by_town[a].iter().zip(&by_town[c]).filter(|(x, y)| x.is_finite() && y.is_finite()).count();
            if both >= 40 {
                pair.push(corr(&by_town[a], &by_town[c]));
            }
        }
    }
    put("mill_towns_pair_corr", mean(&pair));
    let whole_share: Vec<f64> = (k1800..n)
        .map(|k| {
            let u: Vec<f64> = (0..T).filter_map(|t| taken(k, t, mill)).collect();
            u.iter().filter(|&&x| x >= 0.9).count() as f64 / u.len().max(1) as f64
        })
        .collect();
    put("mill_towns_whole_share_sd", sd(&whole_share));
    put("mill_towns_whole_share_max", whole_share.iter().copied().fold(f64::NAN, f64::max));
    // the island's mills: what they took of the island's room, and timber the year after
    let isl_mill: Vec<f64> = (0..n)
        .map(|k| (0..T).map(|t| run.orders[k].added[t * M + mill]).sum::<f64>() / (0..T).map(|t| run.orders[k].budget[t * M + mill]).sum::<f64>().max(1e-9))
        .collect();
    put("mill_island_room_sd", sd(&isl_mill[k1800..]));
    put("mill_room_vs_timber_next_corr", corr(&isl_mill[k1800..n - 1], &dlog(&p_timber)[k1800..]));
    put("mill_room_vs_timber_same_corr", corr(&isl_mill[k1800 + 1..], &dlog(&p_timber)[k1800..]));
    put("mill_room_period", period(&isl_mill[k1800..].iter().map(|x| x - mean(&isl_mill[k1800..])).collect::<Vec<_>>()));
    // the brickworks' room taken on the island, beside the mills' the same year and the year before
    let brick = mi("brickworks");
    let isl_brick: Vec<f64> = (0..n)
        .map(|k| (0..T).map(|t| run.orders[k].added[t * M + brick]).sum::<f64>() / (0..T).map(|t| run.orders[k].budget[t * M + brick]).sum::<f64>().max(1e-9))
        .collect();
    put("brick_room_vs_mill_same_corr", corr(&isl_brick[k1800..], &isl_mill[k1800..]));
    put("brick_room_vs_mill_before_corr", corr(&isl_brick[k1800 + 1..], &isl_mill[k1800..n - 1]));
    // mills' return on their plant and how hard they ran: profit over capacity at plant prices
    let unit_cap = |k: usize, m: usize| -> f64 { (0..G).map(|g| C.m_cap[(m, g)] * series(&format!("price_{}", C.good_name[g]))[k]).sum() };
    let mill_roi: Vec<f64> = (0..n).map(|k| method(k, mill, mcol("profit")) / (method(k, mill, mcol("capacity")) * unit_cap(k, mill)).max(1e-9)).collect();
    let mill_util: Vec<f64> = (0..n).map(|k| method(k, mill, mcol("out")) / (12.0 * method(k, mill, mcol("capacity"))).max(1e-9)).collect();
    put("mill_roi_sd", sd(&mill_roi[k1800..]));
    put("mill_util_mean", mean(&mill_util[k1800..]));
    // the review's own view: orders at the end of year k are decided on year k's profit and prices
    for (j, name) in TRACED.iter().enumerate() {
        let rv: Vec<Review> = run.review.iter().map(|y| y[j]).collect();
        let pick = |f: fn(&Review) -> f64| -> Vec<f64> { rv[k1800..].iter().map(f).collect() };
        put(&format!("review_{name}_grow"), mean(&pick(|r| r.grow)));
        put(&format!("review_{name}_building"), mean(&pick(|r| r.building)));
        put(&format!("review_{name}_low_return"), mean(&pick(|r| r.low)));
        put(&format!("review_{name}_slack"), mean(&pick(|r| r.slack)));
        put(&format!("review_{name}_grow_sd"), sd(&pick(|r| r.grow)));
        put(&format!("review_{name}_grow_vs_plant_price_corr"), corr(&pick(|r| r.grow), &pick(|r| r.plant_price.ln())));
        put(&format!("review_{name}_grow_vs_timber_corr"), corr(&pick(|r| r.grow), &late(&dev(&p_timber))));
        put(&format!("review_{name}_plant_price_swing"), sd(&dlog(&pick(|r| r.plant_price))));
    }
    put("mill_room_vs_roi_corr", corr(&isl_mill[k1800..], &mill_roi[k1800..]));
    put("mill_room_vs_timber_decision_year_corr", corr(&isl_mill[k1800..], &late(&dev(&p_timber))));
    // timber's month-end prices: the dearest month of each year over the year's cheapest month
    let pm: Vec<f64> = st.series.index.get("price_timber").map_or(Vec::new(), |&s| st.series.data[s].clone());
    if pm.len() >= 12 * n {
        let within: Vec<f64> = (k1800..n)
            .map(|k| {
                let y = &pm[12 * k..12 * k + 12];
                y.iter().copied().fold(f64::NAN, f64::max) / y.iter().copied().fold(f64::NAN, f64::min)
            })
            .collect();
        put("timber_month_high_over_low_mean", mean(&within));
        put("timber_month_high_over_low_max", within.iter().copied().fold(f64::NAN, f64::max));
        let med = median(&pm[12 * k1800..12 * n]);
        put("timber_dearest_month_over_median", pm[12 * k1800..12 * n].iter().copied().fold(f64::NAN, f64::max) / med);
        put("timber_months_over_2_medians", pm[12 * k1800..12 * n].iter().filter(|&&p| p > 2.0 * med).count() as f64);
    }

    // ---- (b) tools
    let p_tools = series("price_tools");
    let p_mach = series("price_machinery");
    let mt = median(&p_tools[kfrom..]);
    let dearest = (kfrom..n).max_by(|&a, &c| p_tools[a].total_cmp(&p_tools[c])).unwrap();
    put("tools_dearest_over_median", p_tools[dearest] / mt);
    put("tools_dearest_year", (y0 + dearest as i64) as f64);
    put("tools_years_over_3_medians", (kfrom..n).filter(|&k| p_tools[k] > 3.0 * mt).count() as f64);
    put("machinery_dearest_over_median", p_mach[kfrom..].iter().copied().fold(f64::NAN, f64::max) / median(&p_mach[kfrom..]));
    put("tools_swing", sd(&dlog(&late(&p_tools))));
    let fm = st.tech_year.get(st.t_fmach).copied().unwrap_or(-1);
    put("farm_machinery_year", if fm > 0 { fm as f64 } else { f64::NAN });
    let famine: Vec<usize> = (kfrom..n).filter(|&k| p_tools[k] > 3.0 * mt).collect();
    let calm: Vec<usize> = (kfrom..n).filter(|&k| p_tools[k] <= 1.5 * mt).collect();
    let over = |ks: &[usize], f: &dyn Fn(usize) -> f64| -> f64 { mean(&ks.iter().map(|&k| f(k)).collect::<Vec<_>>()) };
    let unmet = |k: usize, g: usize| 1.0 - bought(k, g) / island(k, g, gcol("asked")).max(1e-9);
    let share = |k: usize, c: &str| island(k, tools, gcol(c)) / bought(k, tools).max(1e-9);
    let smithy_room = |k: usize| (0..T).map(|t| run.orders[k].added[t * M + smithy]).sum::<f64>() / (0..T).map(|t| run.orders[k].budget[t * M + smithy]).sum::<f64>().max(1e-9);
    let smithy_util = |k: usize| method(k, smithy, mcol("out")) / (12.0 * method(k, smithy, mcol("capacity"))).max(1e-9);
    let smithy_roi = |k: usize| method(k, smithy, mcol("profit")) / (method(k, smithy, mcol("capacity")) * unit_cap(k, smithy)).max(1e-9);
    let smithy_growth = |k: usize| if k > 0 { method(k, smithy, mcol("capacity")) / method(k - 1, smithy, mcol("capacity")).max(1e-9) - 1.0 } else { f64::NAN };
    let tools_use = |k: usize, m: usize| -> f64 { b.year(k).unwrap().uses.iter().filter(|u| u.good as usize == tools && u.method as usize == m).map(|u| u.used as f64).sum() };
    let hh = |k: usize| island(k, tools, gcol("households")) + island(k, tools, gcol("landlords")) + island(k, tools, gcol("families"));
    for (label, ks) in [("famine", &famine), ("calm", &calm)] {
        put(&format!("tools_{label}_years"), ks.len() as f64);
        put(&format!("tools_{label}_unmet"), over(ks, &|k| unmet(k, tools)));
        put(&format!("tools_{label}_households_share"), over(ks, &|k| hh(k) / bought(k, tools).max(1e-9)));
        put(&format!("tools_{label}_works_share"), over(ks, &|k| share(k, "works_inputs")));
        put(&format!("tools_{label}_building_share"), over(ks, &|k| share(k, "building")));
        put(&format!("tools_{label}_engine_share"), over(ks, &|k| tools_use(k, engine) / bought(k, tools).max(1e-9)));
        put(&format!("tools_{label}_millwright_share"), over(ks, &|k| tools_use(k, millwright) / bought(k, tools).max(1e-9)));
        put(&format!("tools_{label}_smithy_room"), over(ks, &|k| smithy_room(k)));
        put(&format!("tools_{label}_smithy_util"), over(ks, &|k| smithy_util(k)));
        put(&format!("tools_{label}_smithy_roi"), over(ks, &|k| smithy_roi(k)));
        put(&format!("tools_{label}_smithy_growth"), over(ks, &|k| smithy_growth(k)));
        put(&format!("tools_{label}_bought_per_head"), over(ks, &|k| bought(k, tools) / series("pop")[k].max(1.0)));
        put(&format!("tools_{label}_households_per_head"), over(ks, &|k| hh(k) / series("pop")[k].max(1.0)));
        put(&format!("tools_{label}_machinery_unmet"), over(ks, &|k| unmet(k, machinery)));
        // the same years from 1860 on, so that a famine year is set beside calm years of
        // its own decades and not beside the early century, when households still bought tools
        let late: Vec<usize> = ks.iter().copied().filter(|&k| y0 + k as i64 >= 1860).collect();
        put(&format!("tools_{label}_from_1860_years"), late.len() as f64);
        put(&format!("tools_{label}_from_1860_households_per_head"), over(&late, &|k| hh(k) / series("pop")[k].max(1.0)));
        put(&format!("tools_{label}_from_1860_households_share"), over(&late, &|k| hh(k) / bought(k, tools).max(1e-9)));
    }

    // ---- (c) every good: how it swings, and what might make it
    let h: Vec<f64> = rec.harvest.iter().map(|&v| (v as f64).ln()).collect();
    for g in 0..G {
        let name = &C.good_name[g];
        let p = series(&format!("price_{name}"));
        let d = dlog(&late(&p));
        put(&format!("good_{name}_swing"), sd(&d));
        put(&format!("good_{name}_amp"), sd(&late(&dev(&p))));
        put(&format!("good_{name}_period"), period(&late(&dev(&p))));
        put(&format!("good_{name}_ac1"), ac1(&d));
        put(&format!("good_{name}_dearest"), p[kfrom..].iter().copied().fold(f64::NAN, f64::max) / median(&p[kfrom..]));
        // moving with iron's price, or with timber's, from year to year
        put(&format!("good_{name}_with_iron_corr"), corr(&d, &dlog(&late(&series("price_iron")))));
        put(&format!("good_{name}_with_timber_corr"), corr(&d, &dlog(&late(&p_timber))));
        if h.len() == n {
            // the change of the price on the year's harvest (as a log)
            put(&format!("good_{name}_harvest_corr"), corr(&d, &h[k1800 + 1..]));
        }
        // what was bought for plant, railways, canals, farms and mines, over all bought
        let inv: Vec<f64> = (0..n).map(|k| island(k, g, gcol("building")) + island(k, g, gcol("projects")) + island(k, g, gcol("land"))).collect();
        let all: Vec<f64> = (0..n).map(|k| bought(k, g)).collect();
        put(&format!("good_{name}_capital_share"), (k1800..n).map(|k| inv[k]).sum::<f64>() / (k1800..n).map(|k| all[k]).sum::<f64>().max(1e-9));
        put(&format!("good_{name}_vs_capital_corr"), corr(&late(&dev(&p)), &late(&dev(&inv.iter().map(|x| x.max(1e-9)).collect::<Vec<_>>()))));
        put(&format!("good_{name}_unmet"), mean(&(k1800..n).map(|k| unmet(k, g)).collect::<Vec<_>>()));
        // a thin market: how many towns make it, and how much harder a town's price swings than the island's
        let makers: Vec<f64> = (k1800..n)
            .map(|k| {
                let tot = island(k, g, gcol("made"));
                let y = b.year(k).unwrap();
                (0..T).filter(|&t| y.goods[(t * G + g) * KG + gcol("made")] as f64 > 0.05 * tot && tot > 0.0).count() as f64
            })
            .collect();
        put(&format!("good_{name}_towns_making"), mean(&makers));
        let mut town_sd = (0.0, 0.0);
        for t in 0..T {
            let pt: Vec<f64> = (k1800..n).map(|k| b.year(k).unwrap().goods[(t * G + g) * KG + gcol("price")] as f64).collect();
            let w: f64 = (k1800..n).map(|k| buyers.clone().map(|c| b.year(k).unwrap().goods[(t * G + g) * KG + c] as f64).sum::<f64>()).sum();
            if pt.iter().all(|x| *x > 0.0) && w > 0.0 {
                town_sd.0 += w * sd(&dlog(&pt));
                town_sd.1 += w;
            }
        }
        put(&format!("good_{name}_town_swing"), town_sd.0 / town_sd.1.max(1e-9));
        let stock: Vec<f64> = (k1800..n).map(|k| island(k, g, gcol("stock")) / ((bought(k, g) + island(k, g, gcol("sent"))) / 12.0).max(1e-9)).collect();
        put(&format!("good_{name}_stock_months"), mean(&stock));
        put(&format!("good_{name}_value_share"), mean(&(k1800..n).map(|k| island(k, g, gcol("made")) * p[k] / (12.0 * series("gdp_nom")[k]).max(1e-9)).collect::<Vec<_>>()));
    }

    // ---- the seed year by year
    let mut detail = Vec::new();
    if want_detail {
        detail.push(format!(
            "{:>4}  {:>7} {:>6} {:>5}  {:>6} {:>6} {:>8} {:>7} {:>6}  {:>5} {:>5} {:>5} {:>5} {:>6}   {:>7} {:>6} {:>6} {:>6} {:>6} {:>6} {:>8} {:>8} {:>8} {:>7}",
            "year", "timber", "wage", "cpi", "mills", "whole", "t.ordrd", "t.build", "t.unmt", "m.grw", "m.bld", "m.low", "m.roi", "m.plnt", "tools", "s.room", "s.util", "s.roi", "s.grow", "unmet", "t.hh", "t.works", "t.build", "mach"
        ));
        for k in 0..n {
            let ws: Vec<f64> = (0..T).filter_map(|t| taken(k, t, mill)).collect();
            detail.push(format!(
                "{:>4}  {:7.2} {:6.3} {:5.2}  {:6.2} {:>6} {:8.0} {:7.0} {:6.3}  {:5.2} {:5.2} {:5.2} {:5.2} {:6.2}   {:7.2} {:6.2} {:6.2} {:6.2} {:6.2} {:6.3} {:8.0} {:8.0} {:8.0} {:7.1}",
                y0 + k as i64,
                p_timber[k],
                rw[k],
                cpi[k],
                isl_mill[k],
                format!("{}/{}", ws.iter().filter(|&&x| x >= 0.9).count(), ws.len()),
                ordered_all(k, timber),
                island(k, timber, gcol("building")),
                t_unmet[k],
                run.review[k][0].grow,
                run.review[k][0].building,
                run.review[k][0].low,
                run.review[k][0].roi,
                run.review[k][0].plant_price,
                p_tools[k],
                smithy_room(k),
                smithy_util(k),
                smithy_roi(k),
                smithy_growth(k),
                unmet(k, tools),
                hh(k),
                island(k, tools, gcol("works_inputs")),
                island(k, tools, gcol("building")),
                p_mach[k],
            ));
        }
    }
    Out { nums, detail }
}

// ---------------------------------------------------------------- the table

fn summary(outs: &[(u64, Out)]) -> BTreeMap<String, Vec<f64>> {
    let mut all: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for (_, o) in outs {
        for (k, v) in &o.nums {
            all.entry(k.clone()).or_default().push(*v);
        }
    }
    all
}

fn range(v: &[f64]) -> (f64, f64, f64) {
    let f: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    (mean(&f), f.iter().copied().fold(f64::NAN, f64::min), f.iter().copied().fold(f64::NAN, f64::max))
}

fn fmt(x: f64) -> String {
    if !x.is_finite() {
        "-".into()
    } else if x.abs() >= 1000.0 {
        format!("{x:.0}")
    } else if x.abs() >= 10.0 {
        format!("{x:.1}")
    } else {
        format!("{x:.3}")
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seeds, mut families, mut jobs, mut from) = ((1..=8).collect::<Vec<u64>>(), 6usize, 8usize, 1780i64);
    let (mut sets, mut detail, mut json, mut compare) = (Vec::<(String, String)>::new(), None::<u64>, None::<String>, None::<String>);
    let mut i = 0;
    while i < args.len() {
        let v = args.get(i + 1).cloned().unwrap_or_default();
        match args[i].as_str() {
            "--seeds" => seeds = parse_seeds(&v).expect("seeds"),
            "--families" => families = v.parse().expect("families"),
            "--jobs" => jobs = v.parse().expect("jobs"),
            "--from" => from = v.parse().expect("a year"),
            "--detail" => detail = Some(v.parse().expect("a seed")),
            "--json" => json = Some(v),
            "--compare" => compare = Some(v),
            "--set" => {
                let (k, x) = v.split_once('=').expect("--set key=value");
                sets.push((k.to_string(), x.to_string()));
            }
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    let mut P = Rules::Game.params();
    P.n_families = families;
    P.accounts.enabled = true;
    for (k, x) in &sets {
        P = set_field(&P, k, &parse_value(x)).unwrap_or_else(|e| panic!("{e}"));
    }
    let mut outs: Vec<(u64, Out)> = Vec::new();
    for chunk in seeds.chunks(jobs.max(1)) {
        let got: Vec<(u64, Out)> = std::thread::scope(|s| {
            let hs: Vec<_> = chunk
                .iter()
                .map(|&seed| {
                    let P = &P;
                    s.spawn(move || {
                        let run = play(P, seed);
                        (run.seed, reduce(&run, from, detail == Some(seed)))
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().expect("a run panicked")).collect()
        });
        outs.extend(got);
    }
    println!("game rules, {families} families, seeds {seeds:?}{}", if sets.is_empty() { String::new() } else { format!(", with {}", sets.iter().map(|(k, x)| format!("{k}={x}")).collect::<Vec<_>>().join(" ")) });
    let all = summary(&outs);
    if let Some(f) = &json {
        // a number that could not be had (NaN) is written as null
        std::fs::write(f, serde_json::to_string(&all).unwrap()).expect("write the json");
    }
    let other: Option<BTreeMap<String, Vec<f64>>> = compare.as_ref().map(|f| {
        let read: BTreeMap<String, Vec<Option<f64>>> = serde_json::from_str(&std::fs::read_to_string(f).expect("the file to compare")).expect("a json map");
        read.into_iter().map(|(k, v)| (k, v.into_iter().map(|x| x.unwrap_or(f64::NAN)).collect())).collect()
    });
    println!("{:44} {:>9} {:>20}{}", "", "mean", "[least .. greatest]", if other.is_some() { "      then  [least .. greatest]" } else { "" });
    for (k, v) in &all {
        let (m, lo, hi) = range(v);
        let mut line = format!("{k:44} {:>9} {:>20}", fmt(m), format!("[{} .. {}]", fmt(lo), fmt(hi)));
        if let Some(o) = other.as_ref().and_then(|o| o.get(k)) {
            let (m2, lo2, hi2) = range(o);
            let star = if hi < lo2 || lo > hi2 { " *" } else { "" };
            line += &format!("  {:>9} {:>20}{star}", fmt(m2), format!("[{} .. {}]", fmt(lo2), fmt(hi2)));
        }
        println!("{line}");
    }
    for (seed, o) in &outs {
        if !o.detail.is_empty() {
            println!("\nseed {seed}, year by year: timber's price, the real wage, the cost of living; the mills' room taken on the island and the towns where they took it whole; timber ordered for plant and bought by works being built, and the share of timber asked for and not got; tools' price; the smithies' room taken, how hard they ran, their return on plant and their growth in capacity; tools asked for and not got; tools bought by households, landlords and families, by works as material, by works being built; machinery's price");
            for l in &o.detail {
                println!("{l}");
            }
        }
    }
}
