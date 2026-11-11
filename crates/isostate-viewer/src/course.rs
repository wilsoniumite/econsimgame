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

/// The curve from `a`, leaving along `da`, to `b`, arriving along `db` (both
/// of length one), with a swing to the side, as `n + 1` points.
pub fn curve(a: Vec2, da: Vec2, b: Vec2, db: Vec2, swing: Swing, n: usize) -> Vec<Vec2> {
    let chord = a.distance(b);
    let (ta, tb) = (da * chord * 0.9, db * chord * 0.9);
    let base = |t: f32| -> Vec2 {
        let (t2, t3) = (t * t, t * t * t);
        a * (2.0 * t3 - 3.0 * t2 + 1.0) + ta * (t3 - 2.0 * t2 + t) + b * (-2.0 * t3 + 3.0 * t2) + tb * (t3 - t2)
    };
    let tangent = |t: f32| -> Vec2 {
        let t2 = t * t;
        a * (6.0 * t2 - 6.0 * t) + ta * (3.0 * t2 - 4.0 * t + 1.0) + b * (-6.0 * t2 + 6.0 * t) + tb * (3.0 * t2 - 2.0 * t)
    };
    let n = n.max(1);
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let d = tangent(t).normalize_or(da);
            base(t) + Vec2::new(-d.y, d.x) * swing.at(t)
        })
        .collect()
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
        let (a, b) = (Vec2::new(1.0, 2.0), Vec2::new(2.2, 2.5));
        let (da, db) = (Vec2::X, Vec2::new(0.6, 0.8));
        for swing in [Swing::default(), Swing { bow: 0.2, ess: -0.25, wave: 0.15 }] {
            let pts = curve(a, da, b, db, swing, 200);
            assert!(pts[0].distance(a) < 1e-5 && pts[200].distance(b) < 1e-5);
            // it leaves and arrives along the lines given, whatever the swing
            assert!((pts[1] - pts[0]).normalize().dot(da) > 0.995, "{swing:?}");
            assert!((pts[200] - pts[199]).normalize().dot(db) > 0.995, "{swing:?}");
        }
    }

    #[test]
    fn a_swing_finds_length() {
        let (a, b) = (Vec2::ZERO, Vec2::X);
        let straight = length(&curve(a, Vec2::X, b, Vec2::X, Swing::default(), 100));
        assert!((straight - 1.0).abs() < 1e-3);
        let wound = length(&curve(a, Vec2::X, b, Vec2::X, Swing { bow: 0.0, ess: 0.3, wave: 0.0 }, 100));
        assert!(wound > 1.3, "{wound}");
        let tight = length(&curve(a, Vec2::X, b, Vec2::X, Swing { bow: 0.0, ess: 0.0, wave: 0.28 }, 100));
        assert!(tight > 1.5, "{tight}");
        // and the swing goes no further to the side than it says
        for p in curve(a, Vec2::X, b, Vec2::X, Swing { bow: 0.0, ess: 0.3, wave: 0.0 }, 100) {
            assert!(p.y.abs() < 0.31);
        }
    }

    #[test]
    fn chains_run_between_junctions() {
        // a T: 0-1-2 with 3 hanging off 1
        let adj = vec![vec![1], vec![0, 2, 3], vec![1], vec![1]];
        let mut c = chains(&adj);
        c.sort();
        assert_eq!(c, vec![vec![0, 1], vec![2, 1], vec![3, 1]]);
    }
}
