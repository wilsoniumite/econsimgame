//! Where a railway passes each cell on the new land (the design's item 3,
//! docs/probes/dx-land-view.md).
//!
//! The simulation says which cells a line runs through, and chose them by the
//! cells' mean heights. On the old island the relief below a cell is the
//! viewer's own, made so that a cell's middle is as good a place to pass as
//! any. On the new land the relief every 500 m is the core's: a cell in the
//! hills holds a valley floor, its two sides and a spur between, 150 m or more
//! from top to bottom, and a line that passes each cell near its middle is
//! over the spur as often as round it.
//!
//! So here a line chooses where in each cell it passes by the ground. Each
//! new cell has a grid of spots it might pass at, and a line of cells takes
//! the spots that cost least together (`choose`): nearest one level from cell
//! to cell, so that it climbs no faster than the ruling gradient; with the
//! least ground standing over the straight way between two spots or falling
//! away under it; clear of lakes and tidewater wherever the cell has dry
//! ground; up a valley's side above its floor and its marsh; and across a
//! river squarely. Cells whose track is laid already stay where they are, as
//! they do on the old island, so a branch added later moves nothing.
//!
//! What the simulation does not know, and this cannot mend: it chose the
//! cells. Where every way through a cell is over water or over a ridge, the
//! line is over it.

use std::collections::{HashMap, HashSet};

use bevy::math::Vec2;

use crate::profile::Rules;
use crate::world::{self, Ground};

/// Spots tried along each side of a cell: 267 m apart, which is about as
/// fine as the land's heights are known (a node every 500 m, a vertex of the
/// ground's mesh every 250 m).
const TRY: usize = 7;
/// How far from the cell's edge the spots tried keep, in cells: two cells'
/// spots are then never closer than twice this.
const INSET: f32 = 0.1;
/// Squares along a cell's side that the ground is read at for the way
/// between two spots (125 m).
const FINE: usize = 16;

/// What a spot under standing water (the sea, a tideway, a lake) costs, for
/// each cell of length: so much that a line goes a long way round first.
const WATER: f32 = 6.0;
/// Ground too low to build on (the tide's edge), a valley's floor, marsh: for each cell of length.
const LOW: f32 = 0.5;
const FLOOR: f32 = 0.05;
const MARSH: f32 = 0.25;
/// Ground standing over the way (three times what ground falling away under
/// it costs, as in `rails::draft`: the ground cannot be dug), and what more a
/// tunnel or a viaduct costs than a cutting or a bank, for each cell of length.
const CUT: f32 = 3.0;
const BORED: f32 = 0.3;
const ARCHED: f32 = 0.2;
/// A river crossed: squarely, and the more the further from square.
const CROSSING: f32 = 0.03;
const ASKEW: f32 = 0.3;

/// The ground of one cell as a way finds it, at the middles of `FINE` by `FINE` squares.
struct Patch {
    /// the ground a way would lie on (`rails::lie`)
    ground: Vec<f32>,
    /// what lying there costs besides its earthworks, for each cell of length
    extra: Vec<f32>,
}

/// The lie of the land for a way: what `choose` reads. Each cell's ground is
/// read the first time a line comes to it and kept, and the rivers are marked
/// once on a grid of the whole map.
pub struct Lie {
    w: usize,
    h: usize,
    patches: Vec<Option<Box<Patch>>>,
    /// per square of the whole map (`FINE` to a cell's side): 0, or the way a
    /// river runs there (its bearing, 1 to 250 over a half turn)
    river: Vec<u8>,
}

impl Lie {
    pub fn new(g: &Ground) -> Lie {
        let (w, h) = (g.w, g.h);
        let (fw, fh) = (w * FINE, h * FINE);
        let mut river = vec![0u8; fw * fh];
        let f = FINE as f32;
        for rv in &g.rivers {
            for i in 0..rv.pts.len().saturating_sub(1) {
                // above the tide: below it the river's line is in a tideway, which is standing water
                if rv.level[i] <= 0.004 {
                    continue;
                }
                let (a, b) = (rv.pts[i], rv.pts[i + 1]);
                let d = b - a;
                if d.length_squared() < 1e-12 {
                    continue;
                }
                let bearing = d.y.atan2(d.x).rem_euclid(std::f32::consts::PI) / std::f32::consts::PI;
                let mark = 1 + (bearing * 249.0).round() as u8;
                // a band wide enough that a way read every square does not step over it
                let reach = rv.half[i] + 0.045;
                let (x0, x1) = (((a.x - reach) * f).floor().max(0.0) as usize, (((a.x + reach) * f).floor().max(0.0) as usize).min(fw - 1));
                let (y0, y1) = (((a.y - reach) * f).floor().max(0.0) as usize, (((a.y + reach) * f).floor().max(0.0) as usize).min(fh - 1));
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        if Vec2::new((x as f32 + 0.5) / f, (y as f32 + 0.5) / f).distance(a) <= reach {
                            river[y * fw + x] = mark;
                        }
                    }
                }
            }
        }
        Lie { w, h, patches: (0..w * h).map(|_| None).collect(), river }
    }

    /// Read the ground of the cells about a spot, if they have not been yet.
    fn know(&mut self, g: &Ground, p: Vec2) {
        let (cx, cy) = (p.x.floor() as isize, p.y.floor() as isize);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (x, y) = (cx + dx, cy + dy);
                if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
                    continue;
                }
                let c = y as usize * self.w + x as usize;
                if self.patches[c].is_none() {
                    self.patches[c] = Some(Box::new(patch(g, x as usize, y as usize)));
                }
            }
        }
    }

    /// The ground, what lying there costs besides, and the river's mark, at a spot.
    fn at(&self, g: &Ground, p: Vec2) -> (f32, f32, u8) {
        let f = FINE as f32;
        let (x, y) = ((p.x.max(0.0) as usize).min(self.w - 1), (p.y.max(0.0) as usize).min(self.h - 1));
        let (i, j) = ((((p.x - x as f32) * f).max(0.0) as usize).min(FINE - 1), (((p.y - y as f32) * f).max(0.0) as usize).min(FINE - 1));
        let river = self.river[(y * FINE + j) * self.w * FINE + x * FINE + i];
        match &self.patches[y * self.w + x] {
            Some(patch) => (patch.ground[j * FINE + i], patch.extra[j * FINE + i], river),
            None => {
                let (ground, extra) = footing(g, p.x, p.y);
                (ground, extra, river)
            }
        }
    }
}

/// The ground a way would lie on at a spot, and what lying there costs
/// besides its earthworks, for each cell of length.
fn footing(g: &Ground, x: f32, z: f32) -> (f32, f32) {
    let h = world::height_at(g, x, z);
    let mut extra = if h < 0.0 {
        WATER
    } else if h < 0.05 {
        LOW
    } else {
        0.0
    };
    extra += FLOOR * world::flood_at(g, x, z);
    if let Some(laid) = &g.laid {
        extra += MARSH * world::grid_at(&laid.marsh, g.gw, g.gh, x, z);
    }
    (crate::rails::lie(g, x, z), extra)
}

fn patch(g: &Ground, cx: usize, cy: usize) -> Patch {
    let f = FINE as f32;
    let (mut ground, mut extra) = (Vec::with_capacity(FINE * FINE), Vec::with_capacity(FINE * FINE));
    for j in 0..FINE {
        for i in 0..FINE {
            let (h, e) = footing(g, cx as f32 + (i as f32 + 0.5) / f, cy as f32 + (j as f32 + 0.5) / f);
            ground.push(h);
            extra.push(e);
        }
    }
    Patch { ground, extra }
}

/// A spot a way might pass a cell at.
#[derive(Clone, Copy, Debug)]
struct Spot {
    at: Vec2,
    /// the level the way would have there: the ground's, or the level settled for it
    level: f32,
    /// what passing there costs by itself
    cost: f32,
}

/// What `choose` is told about the way it is placing.
pub struct Asked<'a> {
    pub g: &'a Ground,
    /// the cells' meeting places (`network::centres`)
    pub cen: &'a [(f32, f32)],
    /// the cells the way joins, cell by cell
    pub adj: &'a [Vec<usize>],
    /// the cells of towns: a station stands by the line there, near the market place
    pub towns: &'a HashSet<usize>,
    pub rules: &'a Rules,
    /// a bank higher than this is a viaduct
    pub viaduct: f32,
}

/// The spots a new cell might be passed at, each with what it costs by itself.
fn spots(lie: &Lie, asked: &Asked, c: usize) -> Vec<Spot> {
    let g = asked.g;
    let w = g.w;
    let (cx, cy) = ((c % w) as f32, (c / w) as f32);
    let mid = Vec2::new(asked.cen[c].0, asked.cen[c].1);
    let mut at: Vec<Vec2> = Vec::new();
    if asked.towns.contains(&c) {
        // by a town: round its market place, near enough for a station road and clear of the square
        for (radius, turn) in [(0.18f32, 0.0f32), (0.27, 0.5)] {
            for k in 0..12 {
                let p = mid + Vec2::from_angle((k as f32 + turn) * std::f32::consts::TAU / 12.0) * radius;
                at.push(Vec2::new(p.x.clamp(cx + 0.04, cx + 0.96), p.y.clamp(cy + 0.04, cy + 0.96)));
            }
        }
    } else {
        for j in 0..TRY {
            for i in 0..TRY {
                let along = |k: usize| INSET + (1.0 - 2.0 * INSET) * k as f32 / (TRY - 1) as f32;
                at.push(Vec2::new(cx + along(i), cy + along(j)));
            }
        }
    }
    let mut out: Vec<(Spot, bool)> = at
        .into_iter()
        .map(|p| {
            let (_, extra, river) = lie.at(g, p);
            let (h, ground) = (world::height_at(g, p.x, p.y), crate::rails::lie(g, p.x, p.y));
            let from_mid = p.distance(mid);
            // dry ground, back from a river, off the roads' meeting place and not far from it
            let mut cost = extra + 0.1 * (from_mid - 0.2).abs();
            if river != 0 {
                cost += 0.4;
            }
            if from_mid < 0.1 {
                cost += 0.3;
            }
            (Spot { at: p, level: ground.max(0.03), cost }, h >= 0.05)
        })
        .collect();
    // clear of water and of ground too low to build on wherever the cell has dry ground to pass on
    if out.iter().any(|(_, dry)| *dry) {
        out.retain(|(_, dry)| *dry);
    }
    out.into_iter().map(|(s, _)| s).collect()
}

/// What the straight way between two spots of neighbouring cells costs.
fn between(lie: &Lie, asked: &Asked, a: &Spot, b: &Spot, mids: (Vec2, Vec2)) -> f32 {
    let (g, rules) = (asked.g, asked.rules);
    let d = b.at - a.at;
    let len = d.length();
    // too near each other for a train's curve
    if len < 0.3 {
        return 4.0;
    }
    let dir = d / len;
    let mut cost = 0.15 * len;
    // the climb: what a straight length may climb, what a wound one may, and beyond that an incline
    let rise = (a.level - b.level).abs();
    let (straight, wound) = (rules.cap(len * 1.02, 0), rules.cap(len * 1.02, rules.wind_steps));
    cost += 2.0 * (rise - straight).max(0.0) + 8.0 * (rise - wound).max(0.0);
    if rise > wound {
        cost += 0.8;
    }
    // the ground along it against the way's level
    let n = ((len * FINE as f32).ceil() as usize).max(2);
    let mut in_river = false;
    let mut work = 0.0;
    for i in 1..n {
        let t = i as f32 / n as f32;
        let (ground, extra, river) = lie.at(g, a.at + d * t);
        let dev = ground - (a.level + (b.level - a.level) * t);
        work += extra;
        work += if dev > 0.0 {
            CUT * dev.min(0.3) + if dev > rules.tunnel { BORED } else { 0.0 }
        } else {
            (-dev).min(0.25) + if -dev > asked.viaduct { ARCHED } else { 0.0 }
        };
        // a river is crossed once where the way comes to it, and squarely
        if river != 0 && !in_river {
            let bearing = (river - 1) as f32 / 249.0 * std::f32::consts::PI;
            let with = dir.dot(Vec2::from_angle(bearing));
            cost += CROSSING + ASKEW * with * with;
        }
        in_river = river != 0;
    }
    cost += work * len / n as f32;
    // it keeps to one side of the road between the two meeting places, and runs as the cells do
    let road = mids.1 - mids.0;
    if road.perp_dot(a.at - mids.0) * road.perp_dot(b.at - mids.1) < 0.0 {
        cost += 0.05;
    }
    cost + 0.1 * (1.0 - dir.dot(road.normalize_or(dir)))
}

/// Where a way passes each of its new cells (`fresh`). `settled` tells where
/// it passes a cell whose track is laid already, and its level there.
pub fn choose(lie: &mut Lie, asked: &Asked, fresh: &[usize], settled: &dyn Fn(usize) -> Option<(Vec2, f32)>) -> HashMap<usize, Vec2> {
    let fresh_set: HashSet<usize> = fresh.iter().copied().collect();
    for &c in fresh {
        lie.know(asked.g, Vec2::new(asked.cen[c].0, asked.cen[c].1));
    }
    let lie = &*lie;
    let mid = |c: usize| Vec2::new(asked.cen[c].0, asked.cen[c].1);
    let mut placed: HashMap<usize, Spot> = HashMap::new();
    // the lines of the year's network, those with the most new cells first: a junction is
    // placed by the longest line through it, and the others come to it where it is
    let mut chains = crate::course::chains(asked.adj);
    let news = |cells: &[usize]| cells.iter().filter(|c| fresh_set.contains(c)).count();
    chains.retain(|cells| news(cells) > 0);
    chains.sort_by_key(|cells| std::cmp::Reverse(news(cells)));
    for cells in &chains {
        // a ring has no end to begin from: its first cell takes its own best spot
        if cells.len() > 1 && cells[0] == cells[cells.len() - 1] && fresh_set.contains(&cells[0]) && !placed.contains_key(&cells[0]) {
            if let Some(best) = spots(lie, asked, cells[0]).into_iter().min_by(|a, b| a.cost.total_cmp(&b.cost)) {
                placed.insert(cells[0], best);
            }
        }
        let sets: Vec<Vec<Spot>> = cells
            .iter()
            .map(|&c| match (placed.get(&c), settled(c)) {
                (Some(spot), _) => vec![*spot],
                (None, Some((at, level))) if !fresh_set.contains(&c) => vec![Spot { at, level, cost: 0.0 }],
                _ => spots(lie, asked, c),
            })
            .collect();
        if sets.iter().any(|s| s.is_empty()) {
            continue;
        }
        // the least cost of reaching each spot of each cell along the line, and the spot before it
        let mut best: Vec<Vec<f32>> = vec![sets[0].iter().map(|s| s.cost).collect()];
        let mut from: Vec<Vec<usize>> = vec![vec![0; sets[0].len()]];
        for i in 1..sets.len() {
            let mids = (mid(cells[i - 1]), mid(cells[i]));
            let (mut row, mut came) = (vec![f32::INFINITY; sets[i].len()], vec![0; sets[i].len()]);
            for (j, b) in sets[i].iter().enumerate() {
                for (j0, a) in sets[i - 1].iter().enumerate() {
                    let c = best[i - 1][j0] + between(lie, asked, a, b, mids) + b.cost;
                    if c < row[j] {
                        (row[j], came[j]) = (c, j0);
                    }
                }
            }
            best.push(row);
            from.push(came);
        }
        let last = sets.len() - 1;
        let mut j = (0..sets[last].len()).min_by(|&x, &y| best[last][x].total_cmp(&best[last][y])).unwrap_or(0);
        for i in (0..sets.len()).rev() {
            placed.entry(cells[i]).or_insert(sets[i][j]);
            j = from[i][j];
        }
    }
    // (a cell of track with no neighbour is on no line: its own best spot)
    for &c in fresh {
        if !placed.contains_key(&c) {
            if let Some(best) = spots(lie, asked, c).into_iter().min_by(|a, b| a.cost.total_cmp(&b.cost)) {
                placed.insert(c, best);
            }
        }
    }
    placed.into_iter().filter(|(c, _)| fresh_set.contains(c)).map(|(c, s)| (c, s.at)).collect()
}
