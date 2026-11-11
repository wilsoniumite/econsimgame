//! The course of a way over the map. The simulation says only which cells a
//! railway (or, later, a canal) passes through. Which cells are joined is
//! worked out here (`graph`), the joined cells are strung into lines between
//! junctions (`chains`), and between two cells the way runs on a smooth curve
//! that leaves each cell the way that cell's line points, with a swing to one
//! side or an S or two where it is to follow the hillside or find length
//! (`curve`). Nothing here knows about levels: those are `profile.rs`.

use bevy::math::Vec2;

use crate::replay::Replay;

/// The cells whose transport byte passes `on`, linked to their neighbours (a
/// diagonal only where no pair of straight links joins the two).
pub fn graph(r: &Replay, k: usize, on: impl Fn(u8) -> bool) -> Vec<Vec<usize>> {
    let (w, h, n) = (r.w, r.h, r.n);
    let tr = &r.trans[k * n..(k + 1) * n];
    let has = |x: isize, y: isize| x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && on(tr[y as usize * w + x as usize]);
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for c in 0..n {
        if !on(tr[c]) {
            continue;
        }
        let (x, y) = ((c % w) as isize, (c / w) as isize);
        for (dx, dy) in [(1isize, 0isize), (0, 1), (1, 1), (-1, 1)] {
            let (nx, ny) = (x + dx, y + dy);
            if !has(nx, ny) {
                continue;
            }
            if dx != 0 && dy != 0 && (has(x + dx, y) || has(x, y + dy)) {
                continue;
            }
            let m = ny as usize * w + nx as usize;
            adj[c].push(m);
            adj[m].push(c);
        }
    }
    adj
}

/// The lines as chains of cells between junctions and ends (and closed loops).
pub fn chains(adj: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n = adj.len();
    let mut used = std::collections::HashSet::new();
    let mut out = Vec::new();
    let walk = |start: usize, first: usize, used: &mut std::collections::HashSet<(usize, usize)>| -> Vec<usize> {
        let mut cells = vec![start];
        let (mut prev, mut cur) = (start, first);
        loop {
            used.insert((prev.min(cur), prev.max(cur)));
            cells.push(cur);
            if adj[cur].len() != 2 {
                break;
            }
            let next = if adj[cur][0] == prev { adj[cur][1] } else { adj[cur][0] };
            if used.contains(&(cur.min(next), cur.max(next))) {
                break;
            }
            prev = cur;
            cur = next;
        }
        cells
    };
    for start in 0..n {
        if adj[start].is_empty() || adj[start].len() == 2 {
            continue;
        }
        for &first in &adj[start] {
            if !used.contains(&(start.min(first), start.max(first))) {
                out.push(walk(start, first, &mut used));
            }
        }
    }
    // closed loops, which have no ends
    for start in 0..n {
        for &first in &adj[start] {
            if !used.contains(&(start.min(first), start.max(first))) {
                out.push(walk(start, first, &mut used));
            }
        }
    }
    out
}

/// How a length of way leaves the straight course between its two ends: a
/// bow to one side, an S, and a tighter wave of three, each the furthest it
/// goes to the side in cells. All are nothing at both ends and leave them
/// along the same line, so lengths meet without a kink.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Swing {
    pub bow: f32,
    pub ess: f32,
    pub wave: f32,
}

impl Swing {
    fn at(&self, t: f32) -> f32 {
        let pi = std::f32::consts::PI;
        let s = (pi * t).sin();
        // the S and the wave die away toward the ends
        let ease = |x: f32| {
            let x = (x / 0.25).clamp(0.0, 1.0);
            x * x * (3.0 - 2.0 * x)
        };
        let e = ease(t) * ease(1.0 - t);
        self.bow * s * s + self.ess * (2.0 * pi * t).sin() * e + self.wave * (3.0 * pi * t).sin() * e
    }
}

/// The two ends of a length of way: where each is, and the way the length
/// leaves the first and arrives at the second (both of length one).
#[derive(Clone, Copy, Debug)]
pub struct Ends {
    pub a: Vec2,
    pub da: Vec2,
    pub b: Vec2,
    pub db: Vec2,
}

impl Ends {
    /// The point `t` of the way along (0 to 1), `off` to the side of the smooth curve between the ends.
    pub fn point(&self, t: f32, off: f32) -> Vec2 {
        let chord = self.a.distance(self.b);
        let (a, b, ta, tb) = (self.a, self.b, self.da * chord * 0.9, self.db * chord * 0.9);
        let (t2, t3) = (t * t, t * t * t);
        let base = a * (2.0 * t3 - 3.0 * t2 + 1.0) + ta * (t3 - 2.0 * t2 + t) + b * (-2.0 * t3 + 3.0 * t2) + tb * (t3 - t2);
        let d = (a * (6.0 * t2 - 6.0 * t) + ta * (3.0 * t2 - 4.0 * t + 1.0) + b * (-6.0 * t2 + 6.0 * t) + tb * (3.0 * t2 - 2.0 * t)).normalize_or(self.da);
        base + Vec2::new(-d.y, d.x) * off
    }
}

/// How far to the side the way lies at `t`, from its offsets at evenly
/// spaced stations between the ends (where it is nothing, and leaves along
/// the curve): a smooth line through them.
fn lie_at(lie: &[f32], t: f32) -> f32 {
    if lie.is_empty() {
        return 0.0;
    }
    let k = lie.len() as isize;
    // the stations with the two ends, and one beyond each end that mirrors the first within it
    let get = |i: isize| -> f32 {
        let i = if i < 0 { -i } else if i > k + 1 { 2 * (k + 1) - i } else { i };
        if i == 0 || i == k + 1 { 0.0 } else { lie[(i - 1) as usize] }
    };
    let x = (t.clamp(0.0, 1.0) * (k + 1) as f32).min((k + 1) as f32 - 1e-4);
    let i = x.floor() as isize;
    let u = x - i as f32;
    let (p0, p1, p2, p3) = (get(i - 1), get(i), get(i + 1), get(i + 2));
    0.5 * ((2.0 * p1) + (-p0 + p2) * u + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * u * u + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * u * u * u)
}

/// The curve between two ends with a swing to the side and the offsets of
/// `lie` at evenly spaced stations along it, as `n + 1` points.
pub fn curve(ends: &Ends, swing: Swing, lie: &[f32], n: usize) -> Vec<Vec2> {
    let n = n.max(1);
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            ends.point(t, swing.at(t) + lie_at(lie, t))
        })
        .collect()
}

/// Find how the way should lie between its ends: its offset to the side at
/// each of `stations` evenly spaced stations, no more than `reach` either
/// way, that makes the sum of `cost` along it least. `cost` is asked what a
/// point of the way would cost, given the point and how far along it is (0
/// to 1). The way bends no more sharply from one station to the next than
/// two steps of the nine it may take, and a bend costs a little, so that on
/// plain ground the way runs straight.
pub fn seek(ends: &Ends, swing: Swing, stations: usize, reach: f32, cost: &dyn Fn(Vec2, f32) -> f32) -> Vec<f32> {
    const N: usize = 9;
    if stations == 0 || reach <= 0.0 {
        return vec![0.0; stations];
    }
    let step = 2.0 * reach / (N - 1) as f32;
    let off = |j: usize| -reach + step * j as f32;
    let mid = (N - 1) / 2;
    let t_of = |i: usize| i as f32 / (stations + 1) as f32;
    let at = |t: f32, o: f32| cost(ends.point(t, swing.at(t) + o), t) + 0.01 * o.abs();
    // what it costs to be at each offset at each station, and half way between two stations
    // at each half step (the offset there is the mean of the two)
    let here: Vec<[f32; N]> = (1..=stations).map(|i| std::array::from_fn(|j| at(t_of(i), off(j)))).collect();
    let between: Vec<[f32; 2 * N - 1]> = (0..=stations).map(|i| std::array::from_fn(|h| at((t_of(i) + t_of(i + 1)) * 0.5, -reach + 0.5 * step * h as f32))).collect();
    let bend = |j0: usize, j1: usize| -> f32 {
        let d = (j0 as f32 - j1 as f32).abs();
        if d > 2.0 { f32::INFINITY } else { 2.0 * (d * step) * (d * step) }
    };
    // best[i][j]: the least cost of reaching station i + 1 at offset j
    let mut best: Vec<[f32; N]> = Vec::with_capacity(stations);
    let mut from: Vec<[usize; N]> = Vec::with_capacity(stations);
    for i in 0..stations {
        let mut row = [f32::INFINITY; N];
        let mut came = [mid; N];
        for j in 0..N {
            if i == 0 {
                row[j] = bend(mid, j) + between[0][mid + j] + here[0][j];
                continue;
            }
            for j0 in 0..N {
                let c = best[i - 1][j0] + bend(j0, j) + between[i][j0 + j] + here[i][j];
                if c < row[j] {
                    row[j] = c;
                    came[j] = j0;
                }
            }
        }
        best.push(row);
        from.push(came);
    }
    // and back to the curve at the far end
    let mut j = (0..N).min_by(|&x, &y| (best[stations - 1][x] + bend(x, mid) + between[stations][x + mid]).total_cmp(&(best[stations - 1][y] + bend(y, mid) + between[stations][y + mid]))).unwrap_or(mid);
    let mut lie = vec![0.0; stations];
    for i in (0..stations).rev() {
        lie[i] = off(j);
        j = from[i][j];
    }
    lie
}

/// How long a line of points is.
pub fn length(pts: &[Vec2]) -> f32 {
    pts.windows(2).map(|w| w[0].distance(w[1])).sum()
}

/// The distance along a line of points to each of them.
pub fn along(pts: &[Vec2]) -> Vec<f32> {
    let mut s = Vec::with_capacity(pts.len());
    let mut d = 0.0;
    for (i, p) in pts.iter().enumerate() {
        if i > 0 {
            d += p.distance(pts[i - 1]);
        }
        s.push(d);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_curve_starts_and_ends_where_and_as_it_is_told() {
        let ends = Ends { a: Vec2::new(1.0, 2.0), da: Vec2::X, b: Vec2::new(2.2, 2.5), db: Vec2::new(0.6, 0.8) };
        for (swing, lie) in [(Swing::default(), vec![]), (Swing { bow: 0.2, ess: -0.25, wave: 0.15 }, vec![]), (Swing::default(), vec![0.2, -0.1, 0.3, 0.0, -0.3])] {
            let pts = curve(&ends, swing, &lie, 400);
            assert!(pts[0].distance(ends.a) < 1e-5 && pts[400].distance(ends.b) < 1e-5);
            // it leaves and arrives along the lines given, whatever the swing
            assert!((pts[1] - pts[0]).normalize().dot(ends.da) > 0.995, "{swing:?} {lie:?}");
            assert!((pts[400] - pts[399]).normalize().dot(ends.db) > 0.995, "{swing:?} {lie:?}");
        }
    }

    #[test]
    fn a_swing_finds_length() {
        let ends = Ends { a: Vec2::ZERO, da: Vec2::X, b: Vec2::X, db: Vec2::X };
        let straight = length(&curve(&ends, Swing::default(), &[], 100));
        assert!((straight - 1.0).abs() < 1e-3);
        let wound = length(&curve(&ends, Swing { bow: 0.0, ess: 0.3, wave: 0.0 }, &[], 100));
        assert!(wound > 1.3, "{wound}");
        let tight = length(&curve(&ends, Swing { bow: 0.0, ess: 0.3, wave: 0.2 }, &[], 100));
        assert!(tight > 1.6, "{tight}");
        // and the swing goes no further to the side than it says
        for p in curve(&ends, Swing { bow: 0.0, ess: 0.3, wave: 0.0 }, &[], 100) {
            assert!(p.y.abs() < 0.31);
        }
    }

    #[test]
    fn the_course_lies_where_it_costs_least() {
        let ends = Ends { a: Vec2::ZERO, da: Vec2::X, b: Vec2::X, db: Vec2::X };
        // nothing to choose between: straight
        assert_eq!(seek(&ends, Swing::default(), 5, 0.3, &|_, _| 1.0), vec![0.0; 5]);
        // a hill in the way, a little to one side: round the other side of it
        let hill = |p: Vec2, _: f32| (0.2 - p.distance(Vec2::new(0.5, 0.04))).max(0.0);
        let lie = seek(&ends, Swing::default(), 5, 0.3, &hill);
        assert!(lie[2] < -0.1, "{lie:?}");
        assert!(lie.windows(2).all(|w| (w[0] - w[1]).abs() <= 0.151), "{lie:?}");
        let pts = curve(&ends, Swing::default(), &lie, 60);
        assert!(pts.iter().all(|p| p.distance(Vec2::new(0.5, 0.04)) > 0.1));
    }

    #[test]
    fn chains_run_between_junctions() {
        // a T: 0-1-2 with 3 hanging off 1
        let adj = vec![vec![1], vec![0, 2, 3], vec![1], vec![1]];
        let mut c = chains(&adj);
        c.sort();
        assert_eq!(c, vec![vec![0, 1], vec![1, 2], vec![1, 3]]);
    }
}
