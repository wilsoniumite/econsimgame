//! The traffic record (not in the Python; see docs/rules/traffic.md): what
//! each way carries, kept a year at a time. It only watches.
//!
//! Each year `sim::update_roads` lays the year's carting on the cheapest paths
//! (each district's goods to its market along its town's tree of paths, and the
//! trade between towns along the path from the seller to the buyer), adds it
//! up square by square to wear the roads, and throws it away. This rule walks
//! the same trees with the same weights once more, right after, and adds the
//! weight up on the *links* between neighbouring squares instead, by the way
//! each step of the path went:
//!
//! - **cart**: over the ground, whatever road the square has (none, a track, a
//!   made road, a turnpike);
//! - **boat**: on a navigable river, or on a canal (a step by canal is followed
//!   cell by cell along the canal's own links, `canals::Water::route`);
//! - **rail**: on a railway;
//! - **sea**: between two squares of sea, or from a harbour onto the sea.
//!
//! A link is a square's way to one of four neighbours, east, south, south-east
//! and south-west (`SLOTS`; the other four are its neighbours' links), the same
//! slots the viewer's roads (`network::road_segments`) and the crossings' table
//! use. What is kept of a year: the weight over each link both ways (`goods`),
//! the way most of it went (`mode`), and for the facts the weight times the
//! squares it was carried by each way, the weight sent, and the squares of each
//! road level, railway and canal. Nothing the simulation does reads any of it:
//! with the rule on, every month and every year is played as with it off
//! (`tests/traffic.rs`).
//!
//! The record (`record::pack`) keeps two layers a year: `way_goods`, a byte a
//! link on a scale by ratio (`goods_byte`), and `way_mode`, two bits a link.
//! A live game hands over the same bytes (`live::LiveYear::traffic`).
//!
//! The trade between each pair of towns, which a town's card names its
//! largest partners from, is not kept here: the books keep every flow between
//! two towns, good by good, with its weight and its worth (`accounts::Flow`),
//! and `pairs` sums them up a pair at a time.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::facts::Kind;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::State;
use crate::transport as tr;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TrafficParams {
    /// Keep the traffic record: what each link carried each year, by mode.
    pub enabled: bool,
}

impl Default for TrafficParams {
    /// Off: what the port runs with.
    fn default() -> TrafficParams {
        TrafficParams { enabled: false }
    }
}

impl TrafficParams {
    /// The game's: on.
    pub fn game() -> TrafficParams {
        TrafficParams { enabled: true }
    }
}

/// A square's four links, as (columns east, rows south) to the neighbour:
/// east, south, south-east, south-west.
pub const SLOTS: [(isize, isize); 4] = [(1, 0), (0, 1), (1, 1), (-1, 1)];

/// The ways a load goes, as `mode` keeps them.
pub const CART: u8 = 0;
pub const BOAT: u8 = 1;
pub const RAIL: u8 = 2;
pub const SEA: u8 = 3;
/// Their names, in that order.
pub const MODES: [&str; 4] = ["cart", "boat", "rail", "sea"];

/// The link between neighbouring squares `a` and `b`: the square that holds
/// it and its slot. None if they are not neighbours.
pub fn link(W: usize, a: usize, b: usize) -> Option<(usize, usize)> {
    let (ax, ay) = ((a % W) as isize, (a / W) as isize);
    let (bx, by) = ((b % W) as isize, (b / W) as isize);
    let (dx, dy) = (bx - ax, by - ay);
    let (owner, d) = if dy > 0 || (dy == 0 && dx > 0) { (a, (dx, dy)) } else { (b, (-dx, -dy)) };
    SLOTS.iter().position(|&s| s == d).map(|k| (owner, k))
}

/// The other end of square `c`'s link in slot `k`, if it is on the map.
pub fn other_end(W: usize, H: usize, c: usize, k: usize) -> Option<usize> {
    let (dx, dy) = SLOTS[k];
    let (x, y) = ((c % W) as isize + dx, (c / W) as isize + dy);
    (x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H).then(|| y as usize * W + x as usize)
}

/// The length of a link in slot `k`, in squares (a square is 2 km).
pub fn slot_len(k: usize) -> f64 {
    if k < 2 { 1.0 } else { std::f64::consts::SQRT_2 }
}

/// The least weight a year the record's byte tells from none, and the most it
/// tells apart, as powers of ten: one unit of weight to a thousand million, in
/// 254 steps of 8.5 per cent.
pub const BYTE_LO: f64 = 0.0;
pub const BYTE_HI: f64 = 9.0;

/// The record's byte for a link's weight a year: 0 for less than one unit,
/// else 1 to 255 on a scale by ratio.
pub fn goods_byte(w: f64) -> u8 {
    if !(w >= 10f64.powf(BYTE_LO)) {
        return 0;
    }
    let t = ((w.log10() - BYTE_LO) / (BYTE_HI - BYTE_LO)).clamp(0.0, 1.0);
    1 + (t * 254.0).round() as u8
}

/// The weight a year a byte of the record stands for (the middle of its step).
pub fn byte_goods(b: u8) -> f32 {
    if b == 0 {
        return 0.0;
    }
    10f64.powf(BYTE_LO + (b - 1) as f64 / 254.0 * (BYTE_HI - BYTE_LO)) as f32
}

/// The mode of the link in slot `k` from the record's byte of its square.
pub fn mode_of(byte: u8, k: usize) -> u8 {
    (byte >> (2 * k)) & 3
}

/// One year's sums, for the facts.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TrafficYear {
    pub year: i64,
    /// Weight times the squares it was carried, by mode (`MODES`).
    pub weight_squares: [f64; 4],
    /// The same, of the trade between towns alone.
    pub trade_weight_squares: f64,
    /// The weight sent: each district's goods to its market, and the trade between towns.
    pub sent_district: f64,
    pub sent_trade: f64,
    /// Squares of land by road level, 0 (none) to 3 (a turnpike), as the year's wear left them.
    pub road_squares: [usize; 4],
    /// Squares of land with a railway, and with a canal.
    pub rail_squares: usize,
    pub canal_squares: usize,
}

/// `State::tf`: empty while the rule is off.
#[derive(Clone, Debug, Default)]
pub struct TrafficState {
    /// `[c * 4 + k]`: the weight carried over square c's link in slot k last
    /// year, both ways together.
    pub goods: Vec<f32>,
    /// `[c * 4 + k]`: the way most of that weight went (`CART` to `SEA`).
    pub mode: Vec<u8>,
    /// The weight that left each square on its way last year, or was gathered
    /// at a town's market: what `sim::update_roads` wore the roads with, square
    /// by square, before it put the sea's to nought. Kept for the test that the
    /// two walks are one walk.
    pub through: Vec<f64>,
    /// Every year's sums, from the first recorded.
    pub years: Vec<TrafficYear>,
}

/// The mode of the step from `p` to its neighbour `n` (direction `k` from `p`)
/// in the cost graph the paths were found on.
fn step_mode(st: &State, p: usize, k: usize, n: usize) -> u8 {
    let (g, w) = (&st.graph, &st.world);
    if g.railed.get(p * 8 + k).copied().unwrap_or(false) {
        return RAIL;
    }
    if !w.land[p] || !w.land[n] {
        return SEA;
    }
    if w.navigable[p] && w.navigable[n] && g.w[p * 8 + k] <= tr::RIVER_COST * tr::step_len(k) * (1.0 + 1e-9) {
        return BOAT;
    }
    CART
}

/// The year's traffic (called from `sim::yearly` right after `update_roads`,
/// while the paths are still those the goods took). Does nothing while the
/// rule is off, and in the warm-up.
pub fn year(st: &mut State, P: &Params, year: i64, warm: bool) {
    if !P.traffic.enabled || warm {
        return;
    }
    let (N, W, H, T) = (st.N, st.world.W, st.world.H, st.T_max);
    let mut by = vec![[0.0f64; 4]; N * 4];
    let mut through = vec![0.0f64; N];
    let mut y = TrafficYear { year, ..TrafficYear::default() };
    if st.cell_flow_year.len() == N {
        let boats = st.graph.has_boats();
        let mut water = crate::canals::Water::new(st);
        let st: &State = st;
        // one step of a path: from n to p, its predecessor in town i's tree
        let mut hop = |i: usize, p: usize, n: usize, f: f64, trade: bool, by: &mut Vec<[f64; 4]>, y: &mut TrafficYear| {
            if f <= 0.0 {
                return;
            }
            let mut squares = 0.0;
            if boats && water.afloat(i, p, n) {
                let way = water.route(p, n);
                for s in way.windows(2) {
                    if let Some((c, k)) = link(W, s[0], s[1]) {
                        by[c * 4 + k][BOAT as usize] += f;
                        squares += slot_len(k);
                    }
                }
                y.weight_squares[BOAT as usize] += f * squares;
            } else if let Some(k) = (0..8).find(|&k| tr::neighbour(H, W, p, k) == Some(n)) {
                let m = step_mode(st, p, k, n);
                if let Some((c, s)) = link(W, p, n) {
                    by[c * 4 + s][m as usize] += f;
                }
                squares = tr::step_len(k);
                y.weight_squares[m as usize] += f * squares;
            }
            if trade {
                y.trade_weight_squares += f * squares;
            }
        };
        // each district's goods to its market, as `transport::tree_traffic` lays them
        let mut acc = vec![0.0f64; N];
        for i in 0..T {
            if !st.town_active[i] || st.pred[i].len() != N {
                continue;
            }
            let mut any = false;
            for c in 0..N {
                acc[c] = if st.catch[c] == i as i64 { st.cell_flow_year[c] } else { 0.0 };
                any |= acc[c] > 0.0;
            }
            if !any {
                continue;
            }
            y.sent_district += acc.iter().sum::<f64>();
            for &n in st.settle[i].iter().rev() {
                let n = n as usize;
                through[n] += acc[n];
                let p = st.pred[i][n];
                if p >= 0 {
                    hop(i, p as usize, n, acc[n], false, &mut by, &mut y);
                    acc[p as usize] += acc[n];
                }
            }
        }
        // the trade between towns, as `transport::pair_traffic` lays it
        if st.trade_F_year.rows == T && st.town_cost.rows == T {
            for i in 0..T {
                if st.pred[i].len() != N {
                    continue;
                }
                let root = st.town_node[i] as i64;
                for j in 0..T {
                    let f = st.trade_F_year[(i, j)];
                    if f <= 0.0 || i == j || !st.town_cost[(i, j)].is_finite() {
                        continue;
                    }
                    y.sent_trade += f;
                    let mut n = st.town_node[j] as i64;
                    let mut steps = 0;
                    while n != root && n >= 0 && steps < 100_000 {
                        through[n as usize] += f;
                        let p = st.pred[i][n as usize];
                        if p >= 0 {
                            hop(i, p as usize, n as usize, f, true, &mut by, &mut y);
                        }
                        n = p;
                        steps += 1;
                    }
                }
            }
        }
    }
    // the ways as the year's wear left them
    for c in 0..N {
        if !st.world.land[c] {
            continue;
        }
        y.road_squares[st.road[c].min(3) as usize] += 1;
        y.rail_squares += (st.rail.get(c).copied().unwrap_or(0) > 0) as usize;
        y.canal_squares += (st.canal.get(c).copied().unwrap_or(0) > 0) as usize;
    }
    let tf = &mut st.tf;
    tf.goods = by.iter().map(|m| m.iter().sum::<f64>() as f32).collect();
    tf.mode = by.iter().map(|m| (0..4).fold(0usize, |b, j| if m[j] > m[b] { j } else { b }) as u8).collect();
    tf.through = through;
    tf.years.push(y);
}

/// The year's two layers as the record keeps them (`way_goods`, a byte a link,
/// `[c * 4 + k]`; `way_mode`, a byte a square, two bits a link from the lowest),
/// or None while the rule is off or before its first year.
pub fn layers(st: &State, P: &Params) -> Option<(Vec<u8>, Vec<u8>)> {
    if !P.traffic.enabled {
        return None;
    }
    let N = st.N;
    let tf = &st.tf;
    if tf.goods.len() != N * 4 {
        return Some((vec![0; N * 4], vec![0; N]));
    }
    let goods: Vec<u8> = tf.goods.iter().map(|&w| goods_byte(w as f64)).collect();
    let mode: Vec<u8> = (0..N).map(|c| (0..4).fold(0u8, |b, k| b | ((tf.mode[c * 4 + k] & 3) << (2 * k)))).collect();
    Some((goods, mode))
}

// ---------------------------------------------------------------- partners

/// What one town sold another in a year, summed over the goods.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Pair {
    pub from: u16,
    pub to: u16,
    /// At the selling town's prices.
    pub value: f32,
    /// In units of weight (each good's quantity times its weight).
    pub weight: f32,
    /// The three goods worth most in it, most first; `NO_GOOD` where fewer.
    pub lead: [u16; 3],
}

pub const NO_GOOD: u16 = u16::MAX;

/// The flows between towns of one year (seller, buyer, good, quantity, worth,
/// as the books keep them) summed a pair of towns at a time, in the order of
/// the seller and then the buyer.
pub fn pairs(flows: impl IntoIterator<Item = (u16, u16, u16, f32, f32)>, weight: &[f64]) -> Vec<Pair> {
    use std::collections::BTreeMap;
    let mut acc: BTreeMap<(u16, u16), (f64, f64, BTreeMap<u16, f64>)> = BTreeMap::new();
    for (from, to, good, qty, value) in flows {
        if from == to || !(qty > 0.0 || value > 0.0) {
            continue;
        }
        let e = acc.entry((from, to)).or_default();
        e.0 += value as f64;
        e.1 += qty as f64 * weight.get(good as usize).copied().unwrap_or(0.0);
        *e.2.entry(good).or_default() += value as f64;
    }
    acc.into_iter()
        .map(|((from, to), (value, weight, goods))| {
            let mut g: Vec<(u16, f64)> = goods.into_iter().collect();
            g.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            let mut lead = [NO_GOOD; 3];
            for (l, (good, _)) in lead.iter_mut().zip(g) {
                *l = good;
            }
            Pair { from, to, value: value as f32, weight: weight as f32, lead }
        })
        .collect()
}

/// The pairs of the books' year `year` (empty without the books, or before the year has closed).
pub fn year_pairs(st: &State, C: &Content, year: i64) -> Vec<Pair> {
    let b = &st.acct;
    if !b.on || year < b.start_year {
        return Vec::new();
    }
    let Some(rec) = b.year((year - b.start_year) as usize) else { return Vec::new() };
    pairs(rec.trade.iter().map(|f| (f.from, f.to, f.good, f.qty, f.value)), &C.weight)
}

// ---------------------------------------------------------------- facts

/// The rule's facts at the marks (`facts::marks`): freight carried by each
/// mode in millions of weight-kilometres, the mean haul of everything sent and
/// of the trade between towns in kilometres, and the kilometres of track, made
/// road and turnpike (a square is 2 km). Nothing while the rule is off.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (st, C, P) = (&r.state, &r.content, &r.params);
    let mut out = Vec::new();
    if !P.traffic.enabled {
        return out;
    }
    let y0 = C.start_year;
    let y1 = y0 + r.rec.n_years as i64 - 1;
    for y in crate::facts::marks(y0, y1) {
        let Some(t) = st.tf.years.iter().find(|t| t.year == y) else { continue };
        for (m, name) in MODES.iter().enumerate() {
            out.push((format!("freight_{name}_{y}"), Kind::Real, t.weight_squares[m] * 2.0 / 1e6));
        }
        let sent = t.sent_district + t.sent_trade;
        let all: f64 = t.weight_squares.iter().sum();
        out.push((format!("haul_km_{y}"), Kind::Real, if sent > 0.0 { all * 2.0 / sent } else { f64::NAN }));
        out.push((format!("trade_haul_km_{y}"), Kind::Real, if t.sent_trade > 0.0 { t.trade_weight_squares * 2.0 / t.sent_trade } else { f64::NAN }));
        for (lv, name) in [(1, "track"), (2, "made_road"), (3, "turnpike")] {
            out.push((format!("{name}_km_{y}"), Kind::Count, t.road_squares[lv] as f64 * 2.0));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_link_is_held_by_one_square_and_found_from_either_end() {
        let (W, H) = (5, 4);
        for a in 0..W * H {
            for k in 0..4 {
                let Some(b) = other_end(W, H, a, k) else { continue };
                assert_eq!(link(W, a, b), Some((a, k)));
                assert_eq!(link(W, b, a), Some((a, k)));
            }
        }
        assert_eq!(link(W, 0, 2), None);
        assert_eq!(link(W, 7, 7), None);
    }

    #[test]
    fn the_byte_keeps_a_weight_to_within_its_step() {
        assert_eq!(goods_byte(0.0), 0);
        assert_eq!(goods_byte(0.5), 0);
        assert_eq!(goods_byte(f64::NAN), 0);
        assert_eq!(goods_byte(1.0), 1);
        assert_eq!(goods_byte(1e12), 255);
        for w in [1.0, 3.7, 150.0, 9_000.0, 40_000.0, 2.5e6, 7e8] {
            let back = byte_goods(goods_byte(w)) as f64;
            assert!((back / w).ln().abs() <= 0.5 * (BYTE_HI - BYTE_LO) / 254.0 * 10f64.ln() + 1e-6, "{w} came back {back}");
        }
    }

    #[test]
    fn pairs_sum_a_pair_and_name_its_leading_goods() {
        let w = [1.0, 2.0, 0.5, 1.0];
        let flows = [(0, 1, 0, 10.0, 5.0), (0, 1, 1, 4.0, 20.0), (0, 1, 2, 2.0, 1.0), (0, 1, 3, 1.0, 9.0), (1, 0, 2, 8.0, 2.0), (2, 2, 0, 1.0, 1.0)];
        let p = pairs(flows, &w);
        assert_eq!(p.len(), 2);
        assert_eq!((p[0].from, p[0].to, p[0].value, p[0].weight), (0, 1, 35.0, 10.0 + 8.0 + 1.0 + 1.0));
        assert_eq!(p[0].lead, [1, 3, 0]);
        assert_eq!((p[1].from, p[1].to, p[1].lead), (1, 0, [2, NO_GOOD, NO_GOOD]));
    }
}
