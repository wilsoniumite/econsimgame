//! The two starts (not in the Python; `docs/rules/landfall.md`).
//!
//! **The settled island** is the start every run has had: six towns where the
//! land's makers would put them, 135,000 people shared among them by the
//! fertile land round each, and the woods thinned round every one.
//!
//! **Landfall** is one town in a corner of the island, a harbour at the coast,
//! with `people` settlers in it, and nobody and nothing else on the island: the
//! woods are thinned round that town alone, and the rest is the land as the
//! makers left it. The island itself is the same island, made from the same
//! streams; only the sites are chosen afresh after it is made, and nothing is
//! drawn to choose them.
//!
//! The start is a choice, not a rule: it is the settled island under both sets
//! of rules, and landfall only where it is asked for (`--start landfall` on the
//! command line and in the sweep, `--landfall` in the viewer). Asked for by that
//! word, landfall comes with the three rules that make a colony a colony
//! (`landfall`): births that answer to free land and founding at a hamlet's size
//! (`frontier`), and settlers from abroad (`immigration`). Each can be turned off
//! again by its own switch, and `start.landfall` alone (`--set`) is the start
//! with none of them.

use serde::{Deserialize, Serialize};

use crate::ndimage::{self, Mode};
use crate::params::Params;
use crate::world::{MOUNTAINS, World};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StartParams {
    /// One town in a corner in place of the settled island.
    pub landfall: bool,
    /// The settlers the one town begins with.
    ///
    /// 5,000: a colony some years after its first ships, as Pennsylvania was
    /// about 1685 and South Australia about 1838, big enough to hold a market
    /// of its own (from memory, not from the repository). Fewer is less like
    /// a town: the founding rule wants a district of 40 cells, and the works'
    /// table wants hands; the trial of 26 November ran 5,000 to 2050 on three
    /// islands with no run failing.
    pub people: f64,
}

impl Default for StartParams {
    /// The settled island.
    fn default() -> StartParams {
        StartParams { landfall: false, people: 5_000.0 }
    }
}

/// The most a short town's wage rises in a month on the landfall start:
/// 1.06 a month is twice in a year (`hands.short_rise`).
pub const SHORT_RISE: f64 = 1.06;

/// The first year whose freight follows the price level on the landfall start:
/// the first year of the run, so a colony whose money cannot keep up with its
/// people does not cart at the coin price of 1750 (`longrun.freight_from`).
pub const FREIGHT_FROM: i64 = 1750;

/// The names of the two starts, as the command line takes them.
pub const STARTS: [&str; 2] = ["settled", "landfall"];

/// `P` set to a start by its word: "settled" leaves it as it is; "landfall"
/// takes the one town and the three rules that come with it.
pub fn set_start(P: &mut Params, word: &str) -> Result<(), String> {
    match word {
        "settled" => Ok(()),
        "landfall" => {
            landfall(P);
            Ok(())
        }
        w => Err(format!("--start {w:?}: the starts are {}", STARTS.join(" and "))),
    }
}

/// The landfall start and the rules that come with it: births that answer to
/// free land, hamlets, settlers from abroad, the guard on a short town's wage,
/// carriage that follows prices from 1750, pits at the size of their market,
/// land that stays taken, no settlers to a hungry port and ideas by sea.
pub fn landfall(P: &mut Params) {
    P.start.landfall = true;
    P.frontier = crate::frontier::FrontierParams::game();
    P.immigration = crate::immigration::ImmigrationParams::game();
    // the trap (docs/rules/landfall.md): a short town's wage at most doubles in a year
    P.hands.short_rise = SHORT_RISE;
    // carriage that follows prices from 1750, a stopgap (docs/rules/landfall.md,
    // "Carriage that follows prices"); `--set longrun.freight_from=1900` puts it back
    P.longrun.freight_from = FREIGHT_FROM;
    // pits and quarries that work at the size of their market (docs/rules/mines.md,
    // "G. At the size of the market"); `--set mines.glut=false` puts it back
    P.mines.glut = true;
    // ideas by sea (docs/rules/abroad.md): every port has abroad for a neighbour
    P.abroad = crate::abroad::AbroadParams::game();
}

/// The people the start shares among its towns.
pub fn people(P: &Params) -> f64 {
    if P.start.landfall { P.start.people } else { P.pop0 as f64 }
}

/// The landfall site: the best harbour near a corner of the land.
///
/// The land's box is its least rectangle. A cell is near a corner when it is
/// within 12 cells (24 km) of the nearest coastal cell to that corner; the islands
/// are round, so a true corner is sea. Among the coastal cells near a corner,
/// on land below the mountains and three cells from the map's edge, the best
/// is the one the makers' rule for a town would score highest (the fertile land
/// within five cells, navigable water, and on the new land the places towns
/// grew), with a river's mouth counted as the makers count the coast. Nothing is
/// drawn, so the island is the same island.
pub fn site(world: &World) -> Option<usize> {
    let (W, H) = (world.W, world.H);
    let N = W * H;
    let (mut x0, mut x1, mut y0, mut y1) = (W, 0, H, 0);
    for c in 0..N {
        if world.land[c] {
            let (y, x) = (c / W, c % W);
            x0 = x0.min(x);
            x1 = x1.max(x);
            y0 = y0.min(y);
            y1 = y1.max(y);
        }
    }
    if x0 > x1 {
        return None;
    }
    let corners = [(x0, y0), (x1, y0), (x0, y1), (x1, y1)];
    let ok: Vec<bool> = (0..N)
        .map(|c| {
            let (y, x) = (c / W, c % W);
            world.land[c] && world.coast[c] && world.terrain[c] != MOUNTAINS && y >= 3 && y < H - 3 && x >= 3 && x < W - 3
        })
        .collect();
    let dist = |c: usize, k: usize| ((c % W) as f64 - corners[k].0 as f64).hypot((c / W) as f64 - corners[k].1 as f64);
    // the nearest a harbour comes to each corner
    let mut nearest = [f64::INFINITY; 4];
    for c in (0..N).filter(|&c| ok[c]) {
        for (k, n) in nearest.iter_mut().enumerate() {
            *n = n.min(dist(c, k));
        }
    }
    let fert_sum = ndimage::uniform_filter(&world.fertility, H, W, 11, Mode::Reflect);
    let near_nav = ndimage::binary_dilation(&world.navigable, H, W, false);
    let mut best: Option<(f64, usize)> = None;
    for c in (0..N).filter(|&c| ok[c]) {
        if !(0..4).any(|k| dist(c, k) <= nearest[k] + 12.0) {
            continue;
        }
        let made = world.landform.as_ref().map_or(0.0, |lf| lf.site[c] as f64);
        let s = fert_sum[c] + 0.35 * (near_nav[c] as u8 as f64) + 0.25 * (world.river[c] as u8 as f64) + 0.4 * made;
        if best.is_none_or(|(b, _)| s > b) {
            best = Some((s, c));
        }
    }
    best.map(|(_, c)| c)
}

/// The years `wage_in_grain_Y` is given for, where the run has them: ten years
/// after landfall, then every fifty years from 1800.
pub const WAGE_IN_GRAIN_YEARS: [i64; 7] = [1760, 1800, 1850, 1900, 1950, 2000, 2050];

/// The facts the colony's standing check reads (`standing::COLONY`,
/// docs/rules/scorecard.md, "The colony's check"), given for every run of
/// either start so that the two can be set side by side island by island. They
/// are worked out after the run from what it keeps and change nothing.
///
/// - `natural_increase_pct_A_Z`: births less deaths a thousand over ten, per
///   cent a year, the mean of the years A to Z (the ages' reckoning, ages.rs);
/// - `settlers_share_of_growth_A_Z`: the settlers who came from A to Z over
///   the people gained from A to Z (yearly means), with `immigration` on;
/// - `town_second_year`: the first year the island has two towns or more;
/// - `famine_years_A_Z`: the years from A to Z in which a town went short of
///   food (the `famine` news), as `famine_years` counts the whole run;
/// - `wage_in_grain_Y`: the island's mean wage over the price of grain, the
///   year's means, which can be set against another island's;
/// - `town_wage_rise_max_A_Z`: the largest rise of a town's mean wage over the
///   year before, from A to Z, for towns with people in the two years before;
/// - `price_level_3y_max_A_Z` and `_min_`: the most and least the cost of
///   living (the chained one where the run keeps it) came to over three years,
///   the years from A to Z.
pub fn facts(r: &crate::sim::RunResult, ann: &[Vec<f64>]) -> Vec<(String, crate::facts::Kind, f64)> {
    use crate::facts::Kind;
    let (st, rec) = (&r.state, &r.rec);
    let y0 = r.content.start_year;
    let y1 = y0 + rec.n_years as i64 - 1;
    let series = |name: &str| -> &[f64] { st.series.index.get(name).map_or(&[][..], |&k| &ann[k]) };
    let at = |name: &str, y: i64| -> f64 { series(name).get((y - y0) as usize).copied().unwrap_or(f64::NAN) };
    let inside = |a: i64, z: i64| a >= y0 && z <= y1;
    let mut out = Vec::new();
    // births less deaths, per cent a year
    for (a, z) in [(1751, 1800), (1851, 1880)] {
        if !inside(a, z) || series("ages_births_k").is_empty() {
            continue;
        }
        let v: Vec<f64> = (a..=z).map(|y| (at("ages_births_k", y) - at("ages_deaths_k", y)) / 10.0).collect();
        out.push((format!("natural_increase_pct_{a}_{z}"), Kind::Ratio, v.iter().sum::<f64>() / v.len() as f64));
    }
    // the settlers' share of the growth
    if r.params.immigration.enabled {
        for z in [1770, 1850] {
            if !inside(y0, z) {
                continue;
            }
            let came: f64 = st.immigration.years.iter().filter(|&&(y, _)| y <= z).map(|&(_, n)| n).sum();
            let grew = at("pop", z) - at("pop", y0);
            out.push((format!("settlers_share_of_growth_{y0}_{z}"), Kind::Share, if grew > 0.0 { came / grew } else { f64::NAN }));
        }
    }
    // the second town
    let second = (y0..=y1).find(|&y| at("towns", y) >= 1.5);
    out.push(("town_second_year".to_string(), Kind::Year, second.map_or(f64::NAN, |y| y as f64)));
    // hungry years, before 1900 from twenty years after landfall, and after
    let mut hungry: Vec<i64> = r.events.iter().filter(|e| e.kind == "famine").map(|e| e.year.floor() as i64).collect();
    hungry.dedup();
    for (a, z) in [(1770, 1900), (1901, 2050)] {
        if inside(a, z) {
            out.push((format!("famine_years_{a}_{z}"), Kind::Count, hungry.iter().filter(|&&y| (a..=z).contains(&y)).count() as f64));
        }
    }
    // the wage in grain
    for y in WAGE_IN_GRAIN_YEARS.into_iter().filter(|&y| inside(y, y)) {
        out.push((format!("wage_in_grain_{y}"), Kind::Real, at("wage", y) / at("price_grain", y)));
    }
    // the largest rise of a town's wage in a year, to 1900
    let T = rec.T;
    let z = y1.min(1900);
    let mut most = f64::NAN;
    for k in 2..=((z - y0).max(0) as usize).min(rec.n_years.saturating_sub(1)) {
        for t in 0..T {
            let lived = (k - 2..=k).all(|j| rec.town_active[j * T + t] == 1 && rec.town_pop[j * T + t] > 0.0);
            let (w0, w1) = (rec.town_wage[(k - 1) * T + t] as f64, rec.town_wage[k * T + t] as f64);
            if lived && w0 > 0.0 && w1.is_finite() {
                most = if most.is_nan() { w1 / w0 } else { most.max(w1 / w0) };
            }
        }
    }
    out.push((format!("town_wage_rise_max_{}_{z}", y0 + 2), Kind::Ratio, most));
    // the cost of living over three years, to 1900
    let col = if series("cpi_chained").is_empty() { series("cpi") } else { series("cpi_chained") };
    let n = ((z - y0 + 1).max(0) as usize).min(col.len());
    let swings: Vec<f64> = (0..n.saturating_sub(3)).map(|k| col[k + 3] / col[k]).filter(|v| v.is_finite()).collect();
    if !swings.is_empty() {
        out.push((format!("price_level_3y_max_{y0}_{z}"), Kind::Ratio, swings.iter().copied().fold(f64::NEG_INFINITY, f64::max)));
        out.push((format!("price_level_3y_min_{y0}_{z}"), Kind::Ratio, swings.iter().copied().fold(f64::INFINITY, f64::min)));
    }
    out
}

/// The island made ready for landfall: the one town's site in place of the
/// makers' sites, and the woods of 1750 thinned round it alone, as the makers
/// thin them round every town they place.
pub fn settle(world: &mut World) {
    let Some(c) = site(world) else { return };
    world.town_sites = vec![c];
    let W = world.W;
    world.forest0 = (0..world.W * world.H)
        .map(|i| {
            let dist = ((i / W) as f64 - (c / W) as f64).hypot((i % W) as f64 - (c % W) as f64);
            world.forest_pot[i] * (1.0 - 0.85 * (-dist / 9.0).exp())
        })
        .collect();
}
