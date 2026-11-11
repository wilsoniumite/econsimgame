//! The level of a way. A way has one level at every cell it passes through,
//! settled when the cell's length of it is first laid and kept from then on,
//! so that every length that meets there meets at one level, at a junction as
//! anywhere else. Between cells the level runs on a smooth curve, raised
//! where the ground stands above it (`raise`).
//!
//! `settle` is the rule for a way that climbs by gradients, a railway: no
//! length between two cells may climb more than its length allows, the way
//! lies on the ground or above it (a bank, a viaduct) and under it only in a
//! tunnel, and where the ground is too steep for that a length first finds
//! more length, and at the last is an incline. A canal's rule would be
//! another function over the same nodes and edges (level pounds, and locks
//! where a railway has gradients); `spread`, `hermite` and `raise` are what
//! the two share.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// The rules a graded way keeps. Heights and lengths are in cells.
#[derive(Clone, Copy, Debug)]
pub struct Rules {
    /// the steepest an ordinary length may climb, in height for each cell of length
    pub ruling: f32,
    /// the share of that the levels of two neighbouring cells may use (the
    /// curve between them is steeper in its middle than from end to end)
    pub ease: f32,
    /// the steepest an incline may climb
    pub steep: f32,
    /// a length may wind to this many times its straight length, in `wind_steps` steps
    pub wind: f32,
    pub wind_steps: u8,
    /// a cell's level may stand this far above its ground before the way must follow the ground more closely
    pub high: f32,
    /// and this far where the way stands so high for a long stretch (a cell and all within two of it)
    pub bank: f32,
    /// a cell whose level would lie this far under its ground is in a tunnel
    pub tunnel: f32,
}

impl Rules {
    /// The stage of a length that has become an incline.
    pub fn incline(&self) -> u8 {
        self.wind_steps + 1
    }

    /// How many times its straight length a length at this stage runs to.
    pub fn wound(&self, stage: u8) -> f32 {
        if stage >= self.incline() { 1.0 } else { 1.0 + (self.wind - 1.0) * stage as f32 / self.wind_steps.max(1) as f32 }
    }

    /// The most a length may climb from end to end.
    pub fn cap(&self, len: f32, stage: u8) -> f32 {
        if stage >= self.incline() { self.steep * len } else { self.ease * self.ruling * len * self.wound(stage) }
    }
}

/// A cell the way passes through.
#[derive(Clone, Copy, Debug)]
pub struct Node {
    /// the ground where the way passes
    pub ground: f32,
    /// its level, where that is settled already (or is to be the ground's: a station)
    pub fixed: Option<f32>,
    /// whether it may lie in a tunnel (not a station, a junction or the end of a line)
    pub tunnel_ok: bool,
}

/// A length of way between two cells.
#[derive(Clone, Copy, Debug)]
pub struct Edge {
    pub a: usize,
    pub b: usize,
    /// its straight length
    pub len: f32,
    /// 0 straight, up to `wind_steps` wound, then an incline
    pub stage: u8,
    /// laid now: its stage may still be raised
    pub new: bool,
    /// the edges of one length of way (one number for each length) are raised together
    pub group: usize,
    /// the highest stage it may be raised to: a length is made an incline
    /// only where the ground it has to climb from end to end asks for one
    pub top: u8,
}

pub struct Settled {
    pub level: Vec<f32>,
    /// the cells in a tunnel
    pub under: Vec<bool>,
    /// cells left under the ground, or high above it, because nothing more could give
    pub given_up: usize,
}

#[derive(PartialEq)]
struct Reach(f32, usize);

impl Eq for Reach {}

impl Ord for Reach {
    fn cmp(&self, other: &Reach) -> Ordering {
        other.0.total_cmp(&self.0).then(other.1.cmp(&self.1))
    }
}

impl PartialOrd for Reach {
    fn partial_cmp(&self, other: &Reach) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// For every node, the least of `start[d] + the way from d` over all nodes d
/// (a start of infinity is no start), and the edge it came by. `adj` holds,
/// for each node, its neighbours with the cost of the way to each and the
/// edge's number.
pub fn spread(adj: &[Vec<(usize, f32, usize)>], start: &[f32]) -> (Vec<f32>, Vec<Option<usize>>) {
    let mut val = start.to_vec();
    let mut by: Vec<Option<usize>> = vec![None; start.len()];
    let mut heap: BinaryHeap<Reach> = val.iter().enumerate().filter(|(_, v)| v.is_finite()).map(|(i, &v)| Reach(v, i)).collect();
    while let Some(Reach(v, i)) = heap.pop() {
        if v > val[i] {
            continue;
        }
        for &(j, cost, e) in &adj[i] {
            if v + cost < val[j] {
                val[j] = v + cost;
                by[j] = Some(e);
                heap.push(Reach(v + cost, j));
            }
        }
    }
    (val, by)
}

/// Settle the levels of the nodes that have none, and (where `give` is set)
/// how each new edge climbs. `longest_tunnel` is how many nodes on end may
/// lie in one tunnel.
pub fn settle(nodes: &[Node], edges: &mut [Edge], rules: &Rules, longest_tunnel: usize, give: bool) -> Settled {
    let n = nodes.len();
    let mut given_up = vec![false; n];
    let mut no_tunnel = vec![false; n];
    let neg = |v: &[f32]| -> Vec<f32> { v.iter().map(|x| -x).collect() };
    let mut out = Settled { level: nodes.iter().map(|x| x.fixed.unwrap_or(x.ground)).collect(), under: vec![false; n], given_up: 0 };
    for _ in 0..edges.len() * (rules.incline() as usize + 2) + n + 4 {
        // an edge between two settled cells climbs what it must
        for i in 0..edges.len() {
            let e = edges[i];
            if let (true, true, Some(ha), Some(hb)) = (give, e.new, nodes[e.a].fixed, nodes[e.b].fixed) {
                let mut stage = e.stage;
                while (ha - hb).abs() > rules.cap(e.len, stage) + 1e-5 && stage < e.top {
                    stage += 1;
                }
                for other in edges.iter_mut().filter(|o| o.group == e.group) {
                    other.stage = stage;
                }
            }
        }
        let mut adj: Vec<Vec<(usize, f32, usize)>> = vec![Vec::new(); n];
        for (i, e) in edges.iter().enumerate() {
            let cap = rules.cap(e.len, e.stage);
            adj[e.a].push((e.b, cap, i));
            adj[e.b].push((e.a, cap, i));
        }
        let fixed_or = |other: &dyn Fn(usize) -> f32| -> Vec<f32> { (0..n).map(|c| nodes[c].fixed.unwrap_or_else(|| other(c))).collect() };
        // what the settled cells allow: no higher than `hi`, no lower than `lo`
        let (hi, hi_by) = spread(&adj, &fixed_or(&|_| f32::INFINITY));
        let (lo, lo_by) = spread(&adj, &neg(&fixed_or(&|_| f32::NEG_INFINITY)));
        let lo = neg(&lo);
        // the line that cuts through every rise: where it lies deep under a cell, that cell may be in a tunnel
        let (low, _) = spread(&adj, &fixed_or(&|c| nodes[c].ground));
        let deep = |c: usize| low[c].max(lo[c]);
        let mut under: Vec<bool> = (0..n).map(|c| nodes[c].fixed.is_none() && nodes[c].tunnel_ok && !no_tunnel[c] && nodes[c].ground - deep(c) >= rules.tunnel).collect();
        // no tunnel longer than the years could bore
        let mut seen = vec![false; n];
        for c in 0..n {
            if !under[c] || seen[c] {
                continue;
            }
            let mut group = vec![c];
            seen[c] = true;
            let mut i = 0;
            while i < group.len() {
                for &(m, _, _) in &adj[group[i]] {
                    if under[m] && !seen[m] {
                        seen[m] = true;
                        group.push(m);
                    }
                }
                i += 1;
            }
            if group.len() > longest_tunnel {
                for m in group {
                    under[m] = false;
                }
            }
        }
        // the lowest line that lies on or over the ground everywhere else
        let floor: Vec<f32> = (0..n).map(|c| nodes[c].fixed.unwrap_or(if under[c] { deep(c) } else { nodes[c].ground })).collect();
        let (up, up_by) = spread(&adj, &neg(&floor));
        let up = neg(&up);
        let level: Vec<f32> = (0..n).map(|c| nodes[c].fixed.unwrap_or(up[c].min(hi[c]))).collect();
        out.level = level.clone();
        out.under = under.clone();
        if !give {
            break;
        }
        // a tunnel that came out shallow is no tunnel
        if let Some(c) = (0..n).find(|&c| under[c] && nodes[c].ground - level[c] < 0.6 * rules.tunnel) {
            no_tunnel[c] = true;
            continue;
        }
        // the worst of what is wrong, and the way to what causes it
        let mut worst: Option<(f32, usize, [&Vec<Option<usize>>; 2])> = None;
        for c in (0..n).filter(|&c| nodes[c].fixed.is_none() && !given_up[c]) {
            let (v, by) = if lo[c] > hi[c] + 1e-4 {
                (10.0 + lo[c] - hi[c], [&hi_by, &lo_by])
            } else if !under[c] && nodes[c].ground - level[c] > 0.002 {
                (nodes[c].ground - level[c], [&hi_by, &hi_by])
            } else if level[c] - nodes[c].ground > rules.high {
                (level[c] - nodes[c].ground - rules.high, [&up_by, &up_by])
            } else if level[c] - nodes[c].ground > rules.bank && {
                // every node within two of it stands as high
                let over = |m: usize| nodes[m].fixed.is_none() && level[m] - nodes[m].ground > rules.bank;
                adj[c].len() > 1 && adj[c].iter().all(|&(m, _, _)| over(m) && adj[m].iter().all(|&(m2, _, _)| over(m2)))
            } {
                (0.5 * (level[c] - nodes[c].ground - rules.bank), [&up_by, &up_by])
            } else {
                continue;
            };
            if worst.as_ref().map_or(true, |w| v > w.0) {
                worst = Some((v, c, by));
            }
        }
        let Some((_, c, bys)) = worst else { break };
        // along the way from the cell to its cause, the new edge over the steepest ground gives
        let ground = |c: usize| nodes[c].fixed.unwrap_or(nodes[c].ground);
        let mut pick: Option<(f32, usize)> = None;
        for by in bys {
            let mut at = c;
            let mut steps = 0;
            while let Some(e) = by[at] {
                let edge = &edges[e];
                if edge.new && edge.stage < edge.top {
                    let over = (ground(edge.a) - ground(edge.b)).abs() - rules.cap(edge.len, edge.stage);
                    if pick.map_or(true, |p| over > p.0) {
                        pick = Some((over, e));
                    }
                }
                at = if edge.a == at { edge.b } else { edge.a };
                steps += 1;
                if steps > n {
                    break;
                }
            }
        }
        match pick {
            Some((_, e)) => {
                let (group, stage) = (edges[e].group, edges[e].stage + 1);
                for other in edges.iter_mut().filter(|o| o.group == group) {
                    other.stage = stage;
                }
            }
            None => given_up[c] = true,
        }
    }
    out.given_up = given_up.iter().filter(|&&x| x).count();
    out
}

/// The curve from level `h0` to `h1` that leaves with slope `m0` and arrives
/// with slope `m1` (both in height over the whole length), at `u` from 0 to 1.
pub fn hermite(h0: f32, h1: f32, m0: f32, m1: f32, u: f32) -> f32 {
    let (u2, u3) = (u * u, u * u * u);
    h0 * (2.0 * u3 - 3.0 * u2 + 1.0) + m0 * (u3 - 2.0 * u2 + u) + h1 * (-2.0 * u3 + 3.0 * u2) + m1 * (u3 - u2)
}

/// The slope of that curve at `u`, in height over the whole length.
#[cfg(test)]
pub fn hermite_slope(h0: f32, h1: f32, m0: f32, m1: f32, u: f32) -> f32 {
    let u2 = u * u;
    (h1 - h0) * (6.0 * u - 6.0 * u2) + m0 * (3.0 * u2 - 4.0 * u + 1.0) + m1 * (3.0 * u2 - 2.0 * u)
}

/// The lowest line that lies on or above `floor` at every point and climbs no
/// more steeply than `grad`. `s` is the distance along the way to each point.
pub fn raise(s: &[f32], floor: &[f32], grad: f32) -> Vec<f32> {
    let mut out = floor.to_vec();
    for i in 1..out.len() {
        out[i] = out[i].max(out[i - 1] - grad * (s[i] - s[i - 1]));
    }
    for i in (0..out.len().saturating_sub(1)).rev() {
        out[i] = out[i].max(out[i + 1] - grad * (s[i + 1] - s[i]));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> Rules {
        Rules { ruling: 0.085, ease: 0.7, steep: 0.45, wind: 1.6, wind_steps: 4, high: 0.1, bank: 0.07, tunnel: 0.1 }
    }

    /// A line of cells one after another, a cell apart, on this ground.
    fn line(ground: &[f32], fixed: &[usize]) -> (Vec<Node>, Vec<Edge>) {
        let nodes = ground.iter().enumerate().map(|(i, &g)| Node { ground: g, fixed: fixed.contains(&i).then_some(g), tunnel_ok: i > 0 && i + 1 < ground.len() }).collect();
        let edges = (1..ground.len()).map(|i| Edge { a: i - 1, b: i, len: 1.0, stage: 0, new: true, group: i, top: 5 }).collect();
        (nodes, edges)
    }

    fn holds(nodes: &[Node], edges: &[Edge], s: &Settled, r: &Rules) {
        for e in edges {
            assert!((s.level[e.a] - s.level[e.b]).abs() <= r.cap(e.len, e.stage) + 1e-4, "edge {}-{} climbs {} at stage {}", e.a, e.b, (s.level[e.a] - s.level[e.b]).abs(), e.stage);
        }
        for (c, node) in nodes.iter().enumerate() {
            if let Some(h) = node.fixed {
                assert_eq!(s.level[c], h);
            }
        }
    }

    #[test]
    fn raise_is_the_lowest_line_over_the_floor() {
        let s: Vec<f32> = (0..50).map(|i| i as f32 * 0.02).collect();
        let floor: Vec<f32> = s.iter().map(|&x| if (0.4..0.5).contains(&x) { 0.2 } else { 0.01 * (x * 9.0).sin() }).collect();
        let line = raise(&s, &floor, 0.085);
        for i in 0..s.len() {
            assert!(line[i] >= floor[i]);
            if i > 0 {
                assert!((line[i] - line[i - 1]).abs() <= 0.085 * 0.02 + 1e-6);
            }
        }
        // a floor that is gentle enough is its own line
        let gentle: Vec<f32> = s.iter().map(|&x| 0.05 * x).collect();
        assert_eq!(raise(&s, &gentle, 0.085), gentle);
    }

    #[test]
    fn gentle_ground_is_followed() {
        let r = rules();
        let (nodes, mut edges) = line(&[0.10, 0.13, 0.17, 0.2, 0.18], &[0]);
        let s = settle(&nodes, &mut edges, &r, 1, true);
        holds(&nodes, &edges, &s, &r);
        for (c, node) in nodes.iter().enumerate() {
            assert!((s.level[c] - node.ground).abs() < 1e-5);
        }
        assert!(edges.iter().all(|e| e.stage == 0) && s.given_up == 0);
    }

    #[test]
    fn a_dip_is_bridged_and_a_narrow_ridge_bored() {
        let r = rules();
        let (nodes, mut edges) = line(&[0.2, 0.2, 0.12, 0.2, 0.2], &[]);
        let s = settle(&nodes, &mut edges, &r, 1, true);
        holds(&nodes, &edges, &s, &r);
        assert!(s.level[2] > 0.13 && !s.under[2], "the dip is crossed high: {:?}", s.level);
        let (nodes, mut edges) = line(&[0.2, 0.2, 0.45, 0.2, 0.2], &[]);
        let s = settle(&nodes, &mut edges, &r, 1, true);
        holds(&nodes, &edges, &s, &r);
        assert!(s.under[2] && s.level[2] < 0.3, "the ridge is bored: {:?}", s.level);
        assert!((s.level[1] - 0.2).abs() < 1e-5 && (s.level[3] - 0.2).abs() < 1e-5);
        // but not before a tunnel of that length could be bored: then the line climbs over, as an incline if it must
        let (nodes, mut edges) = line(&[0.2, 0.2, 0.45, 0.45, 0.45, 0.2, 0.2], &[]);
        let s = settle(&nodes, &mut edges, &r, 1, true);
        holds(&nodes, &edges, &s, &r);
        assert!(!s.under.iter().any(|&u| u));
        assert!(s.level[3] > 0.44);
    }

    #[test]
    fn a_scarp_first_finds_length_and_then_is_an_incline() {
        let r = rules();
        // a rise of an eighth in one cell: more than a straight length climbs, less than a wound one
        let (nodes, mut edges) = line(&[0.1, 0.1, 0.23, 0.23], &[0, 3]);
        let s = settle(&nodes, &mut edges, &r, 1, true);
        holds(&nodes, &edges, &s, &r);
        assert!(edges.iter().all(|e| e.stage < r.incline()), "{edges:?}");
        assert!(edges.iter().any(|e| e.stage > 0));
        assert!(s.given_up == 0 && s.level[1] >= 0.1 - 2e-3 && s.level[2] >= 0.23 - 2e-3, "{:?}", s.level);
        // a rise of three tenths between two stations a cell apart: an incline
        let (nodes, mut edges) = line(&[0.1, 0.4], &[0, 1]);
        let s = settle(&nodes, &mut edges, &r, 1, true);
        holds(&nodes, &edges, &s, &r);
        assert_eq!(edges[0].stage, r.incline());
        // and a quarry up a hillside from a line that is there already
        let (nodes, mut edges) = line(&[0.1, 0.2, 0.4], &[0]);
        let s = settle(&nodes, &mut edges, &r, 1, true);
        holds(&nodes, &edges, &s, &r);
        assert_eq!(s.given_up, 0);
        assert!(s.level[2] >= 0.4 - 2e-3 && s.level[1] >= 0.2 - 2e-3, "{:?}", s.level);
        assert_eq!(edges[1].stage, r.incline());
    }

    #[test]
    fn nothing_settled_is_moved_and_old_lengths_keep_their_stage() {
        let r = rules();
        let (mut nodes, mut edges) = line(&[0.1, 0.12, 0.3], &[0, 1]);
        nodes[1].fixed = Some(0.15);
        edges[0].new = false;
        let s = settle(&nodes, &mut edges, &r, 1, true);
        assert_eq!((s.level[0], s.level[1]), (0.1, 0.15));
        assert_eq!(edges[0].stage, 0);
        assert!(edges[1].stage > 0);
    }

    #[test]
    fn the_curve_between_two_levels_is_within_the_ruling_gradient() {
        let r = rules();
        // the most two neighbours may differ, with level track at both
        let d = r.cap(1.0, 0);
        let steepest = (0..=100).map(|i| hermite_slope(0.0, d, 0.0, 0.0, i as f32 / 100.0).abs()).fold(0.0, f32::max);
        assert!(steepest <= r.ruling * 1.06, "{steepest}");
        assert!((hermite(0.0, d, 0.0, 0.0, 1.0) - d).abs() < 1e-6 && hermite(0.0, d, 0.0, 0.0, 0.0) == 0.0);
    }
}
