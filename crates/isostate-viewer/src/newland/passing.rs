//! Where a railway passes each cell on the new land, and the way it takes
//! between two cells (the design's item 3, docs/probes/dx-land-view.md).
//!
//! The simulation says which cells a line runs through, and chose them by the
//! cells' mean heights. On the old island the relief below a cell is the
//! viewer's own, made so that a cell's middle is as good a place to pass as
//! any. On the new land the relief every 500 m is the core's: a cell in the
//! hills holds a valley floor, its two sides and a spur between, 150 m or more
//! from top to bottom, and a line that passes each cell near its middle is
//! over the spur as often as round it.
//!
//! So here a new line finds its way through its cells by the ground
//! (`choose`). The ground of the cells is read every 125 m, and the way from
//! one end of the line to the other is the one that costs least, stepping
//! from one such spot to the next and keeping to the line's own cells, in
//! their order, give or take 500 m (`MARGIN`: the room a length's curve has
//! always had to one side of the straight way between two cells). What a way
//! costs is its length, and then:
//!
//! - **what it climbs and falls beyond the ruling gradient.** A line may rise
//!   or fall along its way as fast as its rules allow and no faster; whatever
//!   the ground does beyond that is what a viaduct, a tunnel or an incline
//!   will have to make up. So the way keeps the level it has along a valley's
//!   side, goes round a spur and not over it, and where it must cross a ridge
//!   or a hollow takes the saddle and the valley's head;
//! - **standing water**, so much that a line goes round a lake or a tideway
//!   wherever its cells have dry ground to go round by;
//! - **a valley's floor, marsh, the water's edge and ground too low to build
//!   on**, a little;
//! - **a river crossed**, and the more the further from square.
//!
//! A cell is passed at the middle of the way's run through it (by a town,
//! where the way comes nearest the market place without crossing it; where a
//! line stops, as near the cell's meeting place as the ground makes worth
//! while), and the line runs there as the way does. Cells whose track is laid already
//! stay where they are, as on the old island, so a branch moves nothing.
//! `rails::draft` then lays each length's curve along the way found.
//!
//! What the simulation does not know, and this cannot mend: it chose the
//! cells. Where every way through them is over water or across a ridge, the
//! line is over it.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use bevy::math::Vec2;

use crate::profile::Rules;
use crate::world::{self, Ground};

/// Squares along a cell's side that the ground is read at (125 m): half the
/// step of the ground's mesh, which is as fine as the land's heights are drawn.
const FINE: usize = 16;

/// What a cell of length costs over ground where nothing else is wrong: 1.
/// Everything else is priced against it, as so many cells of way round.
///
/// A unit of height climbed or fallen beyond what the ruling gradient allows:
/// a spur 30 m high in the way (0.045 as drawn at three times) is worth a
/// cell and a half of way round.
const CLIMB: f32 = 16.0;
/// How far outside its cells a way may stray, in squares (500 m): as far as
/// a length's curve has always been free to lie to one side of the straight
/// way between two cells. It lets a line round the end of a lake or the head
/// of a gorge that fills the edge of its cell, and cut the corner where two
/// of its cells meet only at one.
const MARGIN: usize = 4;
/// Standing water (the sea, a tideway, a lake), for each cell of length.
const WATER: f32 = 40.0;
/// Ground too low to build on (the tide's edge), the square next to standing
/// water, marsh, a valley's floor: for each cell of length.
const LOW: f32 = 3.0;
const SHORE: f32 = 2.0;
const MARSH: f32 = 1.5;
const FLOOR: f32 = 0.3;
/// A river crossed squarely, and what more a crossing costs that runs along the river.
const CROSSING: f32 = 0.4;
const ASKEW: f32 = 2.5;
/// By a town the way keeps within this of the market place (a station road's
/// length), or pays for each cell of length by how far beyond it is; and it
/// keeps this far off the square itself.
const TOWN_REACH: f32 = 0.3;
const TOWN_CLEAR: f32 = 0.14;
/// Where a line stops, what each cell of distance from the cell's meeting place costs it.
const END_LEAN: f32 = 1.5;

/// The ground of one cell as a way finds it, at the middles of `FINE` by `FINE` squares.
struct Patch {
    /// the ground a way would lie on (`rails::lie`)
    ground: Vec<f32>,
    /// what lying there costs besides its length and its climbing, for each cell of length
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
                // a band wide enough that a way stepping from square to square does not step over it
                let reach = rv.half[i] + 0.05;
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

    /// Read the ground of a cell and of the cells round it (a way may stray
    /// into their edges), if it has not been yet.
    fn know(&mut self, g: &Ground, c: usize) {
        let (cx, cy) = ((c % self.w) as isize, (c / self.w) as isize);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (x, y) = (cx + dx, cy + dy);
                if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
                    continue;
                }
                let m = y as usize * self.w + x as usize;
                if self.patches[m].is_none() {
                    self.patches[m] = Some(Box::new(patch(g, x as usize, y as usize)));
                }
            }
        }
    }

    /// The ground a way would lie on at a square of the whole map, and what
    /// lying there costs besides for each cell of length.
    fn at(&self, g: &Ground, sx: usize, sy: usize) -> (f32, f32) {
        let k = (sy % FINE) * FINE + sx % FINE;
        match &self.patches[(sy / FINE) * self.w + sx / FINE] {
            Some(patch) => (patch.ground[k], patch.extra[k]),
            // (a cell nobody has asked about: read the one square)
            None => footing(g, (sx as f32 + 0.5) / FINE as f32, (sy as f32 + 0.5) / FINE as f32),
        }
    }

    /// The way a river runs at a square of the whole map, if one does.
    fn river_at(&self, sx: usize, sy: usize) -> Option<Vec2> {
        match self.river[sy * self.w * FINE + sx] {
            0 => None,
            mark => Some(Vec2::from_angle((mark - 1) as f32 / 249.0 * std::f32::consts::PI)),
        }
    }
}

/// The ground a way would lie on at a spot, and what lying there costs
/// besides its length and its climbing, for each cell of length.
fn footing(g: &Ground, x: f32, z: f32) -> (f32, f32) {
    let h = world::height_at(g, x, z);
    let step = 1.0 / FINE as f32;
    let mut e = if h < 0.0 {
        WATER
    } else if h < 0.05 {
        LOW
    } else if [(step, 0.0), (-step, 0.0), (0.0, step), (0.0, -step)].into_iter().any(|(dx, dz)| world::height_at(g, x + dx, z + dz) < 0.0) {
        // at the water's edge: a line's curve cuts the corners of the way it follows, and a
        // way a square back from the water keeps the curve out of it
        SHORE
    } else {
        0.0
    };
    e += FLOOR * world::flood_at(g, x, z);
    if let Some(laid) = &g.laid {
        e += MARSH * world::grid_at(&laid.marsh, g.gw, g.gh, x, z);
    }
    (crate::rails::lie(g, x, z), e)
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
}

/// What `choose` found.
#[derive(Default)]
pub struct Chosen {
    /// each new cell: where the way passes it, and the way it runs there
    pub at: HashMap<usize, (Vec2, Vec2)>,
    /// each pair of neighbouring cells (the lower number first) that a way was
    /// found between: the way, from the one's passing place to the other's
    pub ways: HashMap<(usize, usize), Vec<Vec2>>,
}

#[derive(PartialEq)]
struct Reach(f32, u32);

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

/// The sixteen steps a way may take from a square: to the eight squares
/// round it and the eight a knight's move off.
const STEPS: [(isize, isize); 16] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1), (2, 1), (2, -1), (-2, 1), (-2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2)];

/// The way that costs least through a line of cells, in their order, from
/// the first to the last: each of its spots with which cell of the line it
/// belongs to. `from` and `to` are where the way passes the first and the
/// last cell, where that is settled; an end that is not begins or ends
/// wherever about its cell is cheapest, leaning to the cell's meeting place.
///
/// A way keeps to its cells give or take `MARGIN`: a spot outside them
/// belongs to the cell of the line it is nearest, and a step goes from a
/// cell's spots to its own or to those of the cell before or after it in the
/// line, so the cells are passed in their order whatever the way does at
/// their edges.
fn thread(lie: &Lie, asked: &Asked, cells: &[usize], from: Option<Vec2>, to: Option<Vec2>) -> Vec<(usize, Vec2)> {
    let (w, f) = (lie.w, FINE as f32);
    let (fw, fh) = ((w * FINE) as isize, (lie.h * FINE) as isize);
    let n = cells.len();
    let mids: Vec<Vec2> = cells.iter().map(|&c| Vec2::new(asked.cen[c].0, asked.cen[c].1)).collect();
    let towns: Vec<bool> = cells.iter().map(|c| asked.towns.contains(c)).collect();
    // each cell's squares and the margin round them: where they begin, on the whole map's grid
    let (side, margin) = ((FINE + 2 * MARGIN) as isize, MARGIN as isize);
    let origin = |ci: usize| ((cells[ci] % w * FINE) as isize - margin, (cells[ci] / w * FINE) as isize - margin);
    let size = (side * side) as usize;
    let id_of = |ci: usize, sx: isize, sy: isize| -> usize {
        let (ox, oy) = origin(ci);
        ci * size + ((sy - oy) * side + (sx - ox)) as usize
    };
    let square_of = |id: usize| -> (isize, isize) {
        let (ox, oy) = origin(id / size);
        let k = (id % size) as isize;
        (ox + k % side, oy + k / side)
    };
    let spot = |sx: isize, sy: isize| Vec2::new((sx as f32 + 0.5) / f, (sy as f32 + 0.5) / f);
    // how far a square lies outside a cell of the line, in squares (nought inside it)
    let outside = |ci: usize, sx: isize, sy: isize| -> isize {
        let (ox, oy) = (origin(ci).0 + margin, origin(ci).1 + margin);
        let beyond = |v: isize, lo: isize| (lo - v).max(v - (lo + FINE as isize - 1)).max(0);
        beyond(sx, ox).max(beyond(sy, oy))
    };
    // the cell of the line a square belongs to, among the one a step comes from and the two
    // either side of it: the nearest that has the square within its margin (the earlier, if two are as near)
    let owner = |ci: usize, sx: isize, sy: isize| -> Option<usize> {
        (ci.saturating_sub(1)..=(ci + 1).min(n - 1)).filter(|&c| outside(c, sx, sy) <= margin).min_by_key(|&c| (outside(c, sx, sy), c))
    };
    // what a square costs for each cell of length, besides the length: its ground, and by a
    // town how far it is from the market place
    let dear = |ci: usize, sx: isize, sy: isize| -> f32 {
        let mut e = lie.at(asked.g, sx as usize, sy as usize).1;
        if towns[ci] {
            let d = spot(sx, sy).distance(mids[ci]);
            e += 6.0 * (d - TOWN_REACH).max(0.0) + if d < TOWN_CLEAR { 4.0 } else { 0.0 };
        }
        e
    };
    // what it costs a way to begin or end at a spot of an end cell that is not settled: it
    // leans to the cell's meeting place, and more where the line stops there (the end of a
    // mineral line: what it was built to reach is about the meeting place, but a pit's line
    // that stops in the valley under the pit is a line, and one that climbs to it on arches is not)
    let lean = |ci: usize, p: Vec2| -> f32 { p.distance(mids[ci]) * if asked.adj[cells[ci]].len() == 1 { END_LEAN } else { 0.6 } };
    let nearest = |ci: usize, p: Vec2| -> usize {
        let (ox, oy) = origin(ci);
        let (sx, sy) = (((p.x * f).floor() as isize).clamp(ox.max(0), (ox + side - 1).min(fw - 1)), ((p.y * f).floor() as isize).clamp(oy.max(0), (oy + side - 1).min(fh - 1)));
        id_of(ci, sx, sy)
    };
    let mut best = vec![f32::INFINITY; n * size];
    let mut came = vec![u32::MAX; n * size];
    let mut heap: BinaryHeap<Reach> = BinaryHeap::new();
    match from {
        Some(p) => {
            let id = nearest(0, p);
            best[id] = 0.0;
            heap.push(Reach(0.0, id as u32));
        }
        None => {
            for id in 0..size {
                let (sx, sy) = square_of(id);
                if sx < 0 || sy < 0 || sx >= fw || sy >= fh || owner(0, sx, sy) != Some(0) {
                    continue;
                }
                best[id] = lean(0, spot(sx, sy)) + 0.1 * dear(0, sx, sy);
                heap.push(Reach(best[id], id as u32));
            }
        }
    }
    let goal = to.map(|p| nearest(n - 1, p));
    // (the place a cell was passed in an earlier year can lie in its margin, nearer the cell
    // before it in this line than its own; its square is still the last cell's to arrive at)
    let goal_square = goal.map(square_of);
    while let Some(Reach(cost, id)) = heap.pop() {
        let id = id as usize;
        if cost > best[id] {
            continue;
        }
        if goal == Some(id) {
            break;
        }
        let ci = id / size;
        let (sx, sy) = square_of(id);
        let (ground, here) = (lie.at(asked.g, sx as usize, sy as usize).0, dear(ci, sx, sy));
        let in_river = lie.river_at(sx as usize, sy as usize).is_some();
        for (dx, dy) in STEPS {
            let (tx, ty) = (sx + dx, sy + dy);
            if tx < 0 || ty < 0 || tx >= fw || ty >= fh {
                continue;
            }
            let owned = if goal_square == Some((tx, ty)) && ci + 2 >= n { Some(n - 1) } else { owner(ci, tx, ty) };
            let Some(cj) = owned else { continue };
            let len = ((dx * dx + dy * dy) as f32).sqrt() / f;
            let mut step = len * (1.0 + 0.5 * (here + dear(cj, tx, ty)));
            step += CLIMB * ((lie.at(asked.g, tx as usize, ty as usize).0 - ground).abs() - asked.rules.ruling * len).max(0.0);
            // a river is crossed where the way comes to it, and squarely
            if !in_river {
                if let Some(runs) = lie.river_at(tx as usize, ty as usize) {
                    let with = Vec2::new(dx as f32, dy as f32).normalize().dot(runs);
                    step += CROSSING + ASKEW * with * with;
                }
            }
            let to_id = id_of(cj, tx, ty);
            if cost + step < best[to_id] {
                best[to_id] = cost + step;
                came[to_id] = id as u32;
                heap.push(Reach(cost + step, to_id as u32));
            }
        }
    }
    let end = goal.unwrap_or_else(|| {
        let ending = |id: usize| {
            let (sx, sy) = square_of(id);
            best[id] + lean(n - 1, spot(sx, sy))
        };
        ((n - 1) * size..n * size).min_by(|&a, &b| ending(a).total_cmp(&ending(b))).unwrap_or((n - 1) * size)
    });
    if !best[end].is_finite() {
        return Vec::new();
    }
    let mut way = Vec::new();
    let mut id = end;
    loop {
        let (sx, sy) = square_of(id);
        way.push((id / size, spot(sx, sy)));
        if came[id] == u32::MAX {
            break;
        }
        id = came[id] as usize;
    }
    way.reverse();
    // the ends are where they are settled, not at the middle of the square that holds them
    if let (Some(p), Some(first)) = (from, way.first_mut()) {
        first.1 = p;
    }
    if let (Some(p), Some(last)) = (to, way.last_mut()) {
        last.1 = p;
    }
    way
}

/// Where a way passes each of its new cells (`fresh`) and how it runs there,
/// and the way between each two neighbouring cells of a new length (`links`,
/// the lower number first). `settled` tells where it passes a cell whose
/// track is laid already.
pub fn choose(lie: &mut Lie, asked: &Asked, fresh: &[usize], links: &[(usize, usize)], settled: &dyn Fn(usize) -> Option<Vec2>) -> Chosen {
    let fresh_set: HashSet<usize> = fresh.iter().copied().collect();
    for &c in fresh.iter().chain(links.iter().flat_map(|(a, b)| [a, b])) {
        lie.know(asked.g, c);
    }
    let lie = &*lie;
    let mut out = Chosen::default();
    // where each cell is passed, as far as is known: the cells settled in other years, and the new ones as they are placed
    let passed = |out: &Chosen, c: usize| -> Option<Vec2> { out.at.get(&c).map(|p| p.0).or_else(|| if fresh_set.contains(&c) { None } else { settled(c) }) };
    // the lines of the year's network, those with the most new cells first: a junction is
    // placed by the longest line through it, and the others come to it where it is
    let mut chains = crate::course::chains(asked.adj);
    let news = |cells: &[usize]| cells.iter().filter(|c| fresh_set.contains(c)).count();
    chains.retain(|cells| news(cells) > 0);
    chains.sort_by_key(|cells| std::cmp::Reverse(news(cells)));
    // of each line, each run of cells that are not placed yet, with the placed cell before it
    // and the one after it where there is one: the way is sought from the one to the other
    let mut runs: Vec<Vec<usize>> = Vec::new();
    for cells in &mut chains {
        // (a ring comes back to its first cell: its way is sought as far as the cell before)
        if cells.len() > 2 && cells[0] == cells[cells.len() - 1] {
            cells.pop();
        }
        let mut i = 0;
        while i < cells.len() {
            if !fresh_set.contains(&cells[i]) {
                i += 1;
                continue;
            }
            let mut j = i;
            while j + 1 < cells.len() && fresh_set.contains(&cells[j + 1]) {
                j += 1;
            }
            runs.push(cells[i.saturating_sub(1)..=(j + 1).min(cells.len() - 1)].to_vec());
            i = j + 1;
        }
    }
    for cells in &runs {
        let n = cells.len();
        let way = thread(lie, asked, cells, passed(&out, cells[0]), passed(&out, cells[n - 1]));
        if way.is_empty() {
            continue;
        }
        // where the way passes each cell: the place of that spot along the way
        let mut pass: Vec<usize> = Vec::with_capacity(n);
        for (ci, &c) in cells.iter().enumerate() {
            let (lo, hi) = (way.iter().position(|s| s.0 == ci).unwrap_or(0), way.iter().rposition(|s| s.0 == ci).unwrap_or(0));
            let at = if let Some(p) = passed(&out, c) {
                (lo..=hi).min_by(|&a, &b| way[a].1.distance(p).total_cmp(&way[b].1.distance(p))).unwrap_or(lo)
            } else if asked.towns.contains(&c) {
                // by a town: where the way comes nearest the market place, clear of the square
                let mid = Vec2::new(asked.cen[c].0, asked.cen[c].1);
                // (and not at the water's edge or on a floor if the way has better ground as near)
                let f = FINE as f32;
                let off = |i: usize| {
                    let d = way[i].1.distance(mid);
                    let poor = lie.at(asked.g, (way[i].1.x * f).max(0.0) as usize, (way[i].1.y * f).max(0.0) as usize).1;
                    (if d < TOWN_CLEAR { 1.0 + TOWN_CLEAR - d } else { d }) + 0.05 * poor.min(4.0)
                };
                (lo..=hi).min_by(|&a, &b| off(a).total_cmp(&off(b))).unwrap_or(lo)
            } else if ci == 0 {
                lo
            } else if ci == n - 1 {
                hi
            } else {
                // the middle of its run through the cell itself (of its run by the cell, if it only passes its edge)
                let w = asked.g.w;
                let inside: Vec<usize> = (lo..=hi).filter(|&i| way[i].1.x.floor() as usize == c % w && way[i].1.y.floor() as usize == c / w).collect();
                if inside.is_empty() { (lo + hi) / 2 } else { inside[inside.len() / 2] }
            };
            pass.push(at);
        }
        for (ci, &c) in cells.iter().enumerate() {
            if fresh_set.contains(&c) && !out.at.contains_key(&c) {
                let (a, b) = (pass[ci].saturating_sub(3), (pass[ci] + 3).min(way.len() - 1));
                let dir = (way[b].1 - way[a].1).normalize_or(Vec2::X);
                out.at.insert(c, (way[pass[ci]].1, dir));
            }
            if ci + 1 < n && pass[ci + 1] > pass[ci] {
                let (c, m) = (c, cells[ci + 1]);
                let mut between: Vec<Vec2> = way[pass[ci]..=pass[ci + 1]].iter().map(|s| s.1).collect();
                if c > m {
                    between.reverse();
                }
                out.ways.entry((c.min(m), c.max(m))).or_insert(between);
            }
        }
    }
    // (a cell of track with no neighbour is on no line: its meeting place)
    for &c in fresh {
        out.at.entry(c).or_insert((Vec2::new(asked.cen[c].0, asked.cen[c].1), Vec2::X));
    }
    // a new length between two cells that no line above ran through (both settled, or a ring's last): its own way
    for &(a, b) in links {
        if out.ways.contains_key(&(a, b)) {
            continue;
        }
        let (Some(pa), Some(pb)) = (passed(&out, a), passed(&out, b)) else { continue };
        let way = thread(lie, asked, &[a, b], Some(pa), Some(pb));
        if way.len() > 1 {
            out.ways.insert((a, b), way.into_iter().map(|s| s.1).collect());
        }
    }
    out
}

/// How far a spot is from a way (a line of points).
pub fn off_way(way: &[Vec2], p: Vec2) -> f32 {
    way.windows(2)
        .map(|w| {
            let ab = w[1] - w[0];
            let t = ((p - w[0]).dot(ab) / ab.length_squared().max(1e-9)).clamp(0.0, 1.0);
            p.distance(w[0] + ab * t)
        })
        .fold(f32::MAX, f32::min)
}
