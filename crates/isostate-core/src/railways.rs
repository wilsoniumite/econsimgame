//! The game's railways: branch lines priced on what they save, at a believable
//! cost. Not in the Python (`docs/rules/railways.md`).
//!
//! Under the port the network is built in one rush and stops at about 1,000 km
//! in the 1850s, because its promoters only ever look at pairs of towns and at
//! the 25 busiest cells of the map, and the whole of it costs two to four per
//! cent of a year's output. With `railways.enabled` the promoters of `ways.rs`
//! take over from `rail::plan` and `rail::plan_spurs`:
//!
//! - they also look at every town without track and at the districts that spend
//!   most on carting, and run a branch to each from the nearest point of the
//!   network;
//! - every line is valued on the carting it would save over the whole map;
//! - a cell of line takes `cost_mult` times the port's rails, timber and stone,
//!   and `build_months` to lay;
//! - they read the order book for rails before they price a line, so a crowd of
//!   schemes prices the next one out until the rail mills catch up.
//!
//! This file holds the numbers, the railway's `Way`, and the facts the sweep
//! prints about the network.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::facts::Kind;
use crate::mat::Mat;
use crate::ndimage;
use crate::observer::annual;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::{Event, State};
use crate::transport as tr;
use crate::ways::{self, BuildCost, SERVES_TOWN, SERVES_TRUNK, Way, Words};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RailwayParams {
    /// The game's promoters replace the port's.
    pub enabled: bool,
    /// Rails, timber and stone per cell of line, as a multiple of the port's
    /// (`rail_rails_per_cell` and its two neighbours).
    pub cost_mult: f64,
    /// Months a line takes if the market fills every order (the port's take ten).
    pub build_months: f64,
    /// Promoters price rails at the year's price times one plus the rails all
    /// lines have on order over a year's make of rails.
    pub order_book: bool,
    /// Lines that may be building at once. A backstop, not the pace: at the
    /// height of the rush 21 to 35 lines are building, held there by the money
    /// their towns can raise and the price of rails.
    pub max_projects: usize,
    /// Districts looked at each year: those with the largest carting bill.
    pub districts: usize,
    /// A district is looked at only this many cells or more from track...
    pub branch_gap: f64,
    /// ...and this far from the other districts looked at that year.
    pub district_spacing: f64,
    /// Years a refused branch is left alone.
    pub cool_years: i64,
}

impl Default for RailwayParams {
    /// Off: what the port runs with. The other numbers are the game's, so that
    /// `--set railways.enabled=true` measures the mechanism as it is played.
    fn default() -> RailwayParams {
        RailwayParams {
            enabled: false,
            cost_mult: 5.0,
            build_months: 24.0,
            order_book: true,
            max_projects: 40,
            districts: 8,
            branch_gap: 3.0,
            district_spacing: 6.0,
            cool_years: 5,
        }
    }
}

impl RailwayParams {
    /// The game's railways.
    pub fn game() -> RailwayParams {
        RailwayParams { enabled: true, ..RailwayParams::default() }
    }
}

fn rail_layer(st: &State) -> &[u8] {
    &st.rail
}

fn rail_layer_mut(st: &mut State) -> &mut Vec<u8> {
    &mut st.rail
}

fn rail_book_mut(st: &mut State) -> &mut ways::WayBook {
    &mut st.rw
}

/// The railway as the promoters of `ways.rs` see it. The hurdle, the induced
/// traffic, the know-how and the materials per cell are the port's numbers; the
/// rest are `RailwayParams`.
pub fn way(st: &State, C: &Content, P: &Params) -> Way {
    let R = &P.railways;
    let mut materials = vec![0.0; C.G];
    materials[C.gi("rails")] = R.cost_mult * P.rail_rails_per_cell;
    materials[C.gi("timber")] = R.cost_mult * P.rail_timber_per_cell;
    materials[C.gi("stone")] = R.cost_mult * P.rail_stone_per_cell;
    Way {
        id: ways::RAILWAY,
        words: Words { kind: "railway", plan_kind: "rail_plan", noun: "railway", Noun: "Railway", track: "track", branch: "branch line" },
        run_cost: tr::RAIL_COST,
        materials,
        build_months: R.build_months,
        // transport::rail_build_graph's numbers: slopes, mountains and bridges are dear
        build: BuildCost { lowland: 1.0, hills: 2.2, mountains: 6.0, other: 50.0, river: 1.5, slope: 40.0, laid: 0.15 },
        tech: st.t_rail,
        know: P.rail_know,
        hurdle: P.rail_hurdle,
        induced: P.rail_induced,
        order_book: R.order_book.then(|| C.gi("rails")),
        max_projects: R.max_projects,
        districts: R.districts,
        branch_gap: R.branch_gap,
        district_spacing: R.district_spacing,
        cool_years: R.cool_years,
        layer: rail_layer,
        layer_mut: rail_layer_mut,
        book_mut: rail_book_mut,
    }
}

/// The yearly step (called from `rail::plan` when the mechanism is on).
pub fn plan(st: &mut State, C: &Content, P: &Params, year: i64, pbar: &Mat, events: &mut Vec<Event>) {
    let way = way(st, C, P);
    ways::plan(st, C, P, &way, year, pbar, events);
}

/// The monthly step (called from `rail::progress`): nothing to do unless one of
/// the game's lines is building.
pub fn progress(st: &mut State, C: &Content, P: &Params, year_float: f64, events: &mut Vec<Event>) {
    if !st.projects.iter().any(|pr| pr.way == ways::RAILWAY) {
        return;
    }
    let way = way(st, C, P);
    ways::progress(st, &way, year_float, events);
}

/// Facts about the network for the sweep's table, under "Railways". Those that
/// need the promoters' book have no value in a run with the mechanism off.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (st, C, rec) = (&r.state, &r.content, &r.rec);
    let y0 = C.start_year;
    let y1 = y0 + rec.n_years as i64 - 1;
    let series = |name: &str| st.series.index.get(name).map_or(&[][..], |&k| &st.series.data[k][..]);
    let at = |name: &str, y: i64| annual(series(name), y0, y);
    let mut out: Vec<(String, Kind, f64)> = Vec::new();

    // length by decade (1850, 1870 and the last year are already in the table)
    for y in (1830..=1890).step_by(10).filter(|&y| y != 1850 && y != 1870 && y > y0 && y < y1) {
        out.push((format!("rail_km_{y}"), Kind::Count, at("rail_km", y)));
    }
    // decades from 1840 that ended with more line than they began with
    let decades: Vec<i64> = (1840..y1).step_by(10).filter(|&y| y >= y0 && y + 10 <= y1).collect();
    if !decades.is_empty() {
        let grew = decades.iter().filter(|&&y| at("rail_km", y + 10) > at("rail_km", y)).count();
        out.push((format!("rail_decades_grown_of_{}", decades.len()), Kind::Count, grew as f64));
    }

    // towns with track, from the yearly map of roads and rails (4 marks a railway)
    let (N, T) = (rec.N, rec.T);
    let trans = rec.layer("trans");
    for y in [1860, y1].into_iter().filter(|&y| y >= y0 && y <= y1) {
        let k = (y - y0) as usize;
        let towns: Vec<usize> = (0..T).filter(|&t| rec.town_active[k * T + t] > 0).collect();
        let with = towns.iter().filter(|&&t| trans[k * N + st.town_node[t]] & 4 > 0).count();
        out.push((format!("rail_towns_with_track_{y}"), Kind::Share, with as f64 / towns.len().max(1) as f64));
    }

    // the share of the last year's cell traffic that starts within a cell of track
    let (H, W) = (st.world.H, st.world.W);
    let near = if st.rail.iter().any(|&x| x > 0) {
        let off: Vec<bool> = st.rail.iter().map(|&x| x == 0).collect();
        let d = ndimage::distance_transform_edt(&off, H, W);
        let total: f64 = st.cell_flow_year.iter().sum();
        let by: f64 = (0..st.N).filter(|&c| d[c] < 1.5).map(|c| st.cell_flow_year[c]).sum();
        if total > 0.0 { by / total } else { f64::NAN }
    } else {
        f64::NAN
    };
    out.push((format!("rail_traffic_near_track_{y1}"), Kind::Share, near));

    // the dearest year for rails from 1826 against the mean of 1800 to 1825
    if y0 <= 1800 && y1 > 1826 {
        let base: Vec<f64> = (1800..=1825).map(|y| at("price_rails", y)).filter(|v| v.is_finite()).collect();
        let peak = (1826..=y1).map(|y| at("price_rails", y)).filter(|v| v.is_finite()).fold(f64::NAN, f64::max);
        out.push(("rail_price_peak_vs_1800_1825".into(), Kind::Ratio, peak / (base.iter().sum::<f64>() / base.len().max(1) as f64)));
    }

    // ---- from the promoters' book
    let book = &st.rw;
    let on = !book.years.is_empty();
    let nan_off = |v: f64| if on { v } else { f64::NAN };
    // what the network cost: each line's outlay over the output of the year it opened
    let cost_share: f64 = book
        .lines
        .iter()
        .map(|l| {
            let gdp = 12.0 * at("gdp_nom", (l.opened.floor() as i64).clamp(y0, y1));
            if gdp > 0.0 { l.spent / gdp } else { 0.0 }
        })
        .sum();
    out.push(("rail_cost_in_years_of_output".into(), Kind::Real, nan_off(cost_share)));
    let most = book.years.iter().map(|n| n.building).max().unwrap_or(0);
    out.push(("rail_most_lines_building".into(), Kind::Count, nan_off(most as f64)));
    // how long the slowest line took from its promotion to its opening, and how
    // many at the end have been building for ten years or more (a line short of
    // money waits; nothing abandons it)
    let slowest = book.lines.iter().map(|l| l.opened - l.start as f64).fold(f64::NAN, f64::max);
    out.push(("rail_slowest_line_years".into(), Kind::Real, slowest));
    let stuck = st.projects.iter().filter(|pr| pr.way == ways::RAILWAY && pr.start + 10 <= y1).count();
    out.push(("rail_lines_stuck_at_end".into(), Kind::Count, nan_off(stuck as f64)));
    // fares on the year's traffic over track against the capital laid out, the
    // capital brought to that year's prices by the cost of living
    for y in [1870, y1].into_iter().filter(|&y| y >= y0 && y <= y1) {
        let v = match book.years.iter().find(|n| n.year == y) {
            Some(n) => {
                let cpi_y = at("cpi", y);
                let capital: f64 = book
                    .lines
                    .iter()
                    .filter(|l| l.opened < (y + 1) as f64)
                    .map(|l| l.spent * cpi_y / at("cpi", (l.opened.floor() as i64).clamp(y0, y1)))
                    .sum();
                if capital > 0.0 { n.fares / capital } else { f64::NAN }
            }
            None => f64::NAN,
        };
        out.push((format!("rail_fares_over_capital_{y}"), Kind::Real, v));
    }
    // what the lines serve
    let count = |f: &dyn Fn(u8) -> bool| nan_off(book.lines.iter().filter(|l| f(l.serves)).count() as f64);
    out.push(("rail_lines_trunk".into(), Kind::Count, count(&|s| s == SERVES_TRUNK)));
    out.push(("rail_lines_to_towns".into(), Kind::Count, count(&|s| s == SERVES_TOWN)));
    out.push(("rail_lines_to_farms".into(), Kind::Count, count(&|s| s == 2)));
    out.push(("rail_lines_to_pastures".into(), Kind::Count, count(&|s| s == 3)));
    out.push(("rail_lines_to_quarries".into(), Kind::Count, count(&|s| s == 5 || s == 6)));
    out.push(("rail_lines_to_mines".into(), Kind::Count, count(&|s| s == 7 || s == 8)));
    out.push(("rail_lines_to_works".into(), Kind::Count, count(&|s| s == 9)));
    out.push(("rail_lines_to_woods".into(), Kind::Count, count(&|s| s == 4 || s > 9)));
    let km_all: f64 = book.lines.iter().map(|l| 2.0 * l.new_cells as f64).sum();
    let km_branch: f64 = book.lines.iter().filter(|l| l.serves != SERVES_TRUNK).map(|l| 2.0 * l.new_cells as f64).sum();
    out.push(("rail_km_share_branches".into(), Kind::Share, nan_off(if km_all > 0.0 { km_branch / km_all } else { f64::NAN })));
    out
}
