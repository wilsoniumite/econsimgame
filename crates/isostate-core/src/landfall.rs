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
/// free land, hamlets, settlers from abroad, the guard on a short town's wage
/// and carriage that follows prices from 1750.
pub fn landfall(P: &mut Params) {
    P.start.landfall = true;
    P.frontier = crate::frontier::FrontierParams::game();
    P.immigration = crate::immigration::ImmigrationParams::game();
    // the trap (docs/rules/landfall.md): a short town's wage at most doubles in a year
    P.hands.short_rise = SHORT_RISE;
    // carriage that follows prices from 1750, a stopgap (docs/rules/landfall.md,
    // "Carriage that follows prices"); `--set longrun.freight_from=1900` puts it back
    P.longrun.freight_from = FREIGHT_FROM;
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
