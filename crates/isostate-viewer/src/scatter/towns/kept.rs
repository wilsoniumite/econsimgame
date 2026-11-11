//! Towns on the ground they keep (task A). Under the game's rules a town
//! keeps the ground it builds on, and the record says of every cell of it
//! when it joined, how many live on it and how much of it is works
//! (docs/rules/towns.md). The towns of such a run are laid out from that:
//!
//! - **How many houses.** One to every eight residents of the cell
//!   (`world::PEOPLE_TO_A_HOUSE`, where the reason is), so the houses on the
//!   map follow the people from year to year and from town to town.
//! - **Where.** Only on the cell's built-up closes (`world::town_ground`),
//!   which gather from the side of the town's older ground and along the way
//!   out of it. Streets are laid there and nowhere else: the roads, a lane
//!   where no road yet runs between a cell and the older cell it grew from,
//!   and back lanes and cross streets behind them. Plots stand at fixed
//!   places along the streets, and each has a turn (`Plot::turn`): the place
//!   of its close in the cell's order, and a number of its own. The cell's
//!   houses stand on the plots whose turn has come, so a house, once built,
//!   stays where it is.
//! - **When.** A plot is first built in the year its cell first had the
//!   houses for its turn to come, which the record's history gives
//!   (`Story::year_of`). A cell that was town ground when the record opens
//!   is dated back by the rule the simulation fills it by: three hundred
//!   people the year it joined, and the rest as it aged.
//! - **What kind and how tall.** A plot's house is a cottage of one storey,
//!   long and low on the wider plots of the lanes, and of two once its cell
//!   has grown well past it. On the roads through a town and round its
//!   market place a house that has a neighbour is rebuilt as a townhouse,
//!   the full width of its plot, once the cell has some dozens more houses
//!   than when it was built (in the style of that year), and gets a third
//!   storey only where houses stand on both sides of it and the cell has
//!   grown as much again. The back streets go up a row at a time, brick
//!   terraces from 1815. Villas stand on the lanes of ground that has little
//!   works on it. So frontages join before they rise, and nothing tall
//!   stands alone.
//!
//! A run under the port's rules has no such record and is laid out as before
//! (`houses_port`).
//!
//! **Taken over.** The first helper on this task was cut off and left no
//! account of its decisions. Read back from its code, they were these, and
//! the second helper kept every one of them unless a line below says not:
//!
//! 1. Eight people to a drawn house (`world::PEOPLE_TO_A_HOUSE`): the unit
//!    the record keeps residents in, and as many houses as the fullest
//!    ground of 1900 has room for at the size houses are drawn.
//! 2. A parcel of town ground is cut three times more, into eight closes
//!    (some eighty to a cell), and a close is built up or open.
//! 3. The order a cell's closes are built up in is fixed the year the cell
//!    joins (`world::town_order`): by the way to the market place along the
//!    roads of that year, ground off the road counting three times.
//! 4. As many closes are built up as the cell's houses need at
//!    `world::HOUSE_GROUND` each, and its works for their share of the cell.
//! 5. Houses outlast a fall of a quarter in their cell's residents
//!    (`world::HOUSES_KEEP`).
//! 6. A plot's turn is its close's place in that order plus up to `THIN`
//!    closes of its own, and the cell's houses stand on the plots whose turn
//!    comes first.
//! 7. A house's year is the year the record first shows its cell with that
//!    many houses; ground older than the record is dated back by the rule
//!    the simulation fills it by.
//! 8. Cottages of one storey, then two; townhouses on roads and market
//!    places only where a neighbour stands; a third storey only between two
//!    neighbours; terraces on rows from 1815; villas on lanes of ground with
//!    little works.
//! 9. The open closes are painted by the ground shader as paddocks,
//!    orchards, garden ground and greens within hedges (`town_edge` in
//!    terrain.wesl).

use super::*;

/// How many closes a plot's own number may put its turn back by: the edge
/// of the built-up ground thins out over about this many closes.
const THIN: f32 = 5.0;
/// A third ring of lanes, for cells that fill.
const FAR_LANE: f32 = 0.39;

/// What a year's layout of the kept towns came to, for measuring it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Census {
    /// cells of kept town ground, and the people the record says live on them
    pub cells: usize,
    pub residents: f32,
    /// the houses those people ask for (with those that outlast a fall), and the houses drawn
    pub wanted: usize,
    pub drawn: usize,
    pub cottages: usize,
    pub townhouses: usize,
    pub terraced: usize,
    pub villas: usize,
    /// houses of one, two and three storeys or more
    pub storeys: [usize; 3],
    /// three-storey houses with no neighbour on one side (there should be none)
    pub tall_alone: usize,
}

/// A street that may be laid on kept town ground.
struct Way {
    a: Vec2,
    b: Vec2,
    width: f32,
    id: i32,
    /// a road, which is drawn already
    road: bool,
    /// 0 the way between two cells, 1 to 3 the lanes behind it, 4 a cross street, 5 a station road
    kind: u8,
    /// for a lane or a cross street, the way it belongs to
    main: i32,
}

/// The story of a cell's houses up to this year.
struct Story {
    /// the houses that stand this year
    houses: usize,
    /// each time the cell had more houses than ever before since it joined: how many, and the year (counted from the start)
    steps: Vec<(usize, usize)>,
    /// the year (counted from the start) the cell joined, or 0 if it was town ground when the record opens
    since: usize,
    /// the calendar year it joined
    joined: i32,
    /// the share of it under works
    works: f32,
    /// a town's first cell, with its market place
    node: bool,
}

impl Story {
    fn of(r: &Replay, k: usize, c: usize) -> Story {
        let mut since = k;
        while since > 0 && r.built[(since - 1) * r.n + c] > 0 {
            since -= 1;
        }
        let mut steps: Vec<(usize, usize)> = Vec::new();
        for y in since..=k {
            let n = (r.residents(y, c) / world::PEOPLE_TO_A_HOUSE).round() as usize;
            if steps.last().map_or(true, |s| n > s.0) {
                steps.push((n, y));
            }
        }
        Story {
            houses: world::town_houses(r, k, c),
            steps,
            since,
            joined: r.joined(k, c).unwrap_or(r.y0),
            works: r.works_share(k, c),
            node: r.header.towns.iter().any(|t| t.node == c),
        }
    }

    /// The calendar year the cell first had `n` houses, as far as this year
    /// knows. `own` is a number of the plot's own, 0 to 1. For a house that
    /// stood when the record opens the year is reckoned back as the
    /// simulation fills old ground (`isostate_core::towns::room`): three
    /// hundred people the year the cell joined, the rest as it aged, most of
    /// them early.
    fn year_of(&self, r: &Replay, k: usize, n: usize, own: f32) -> i32 {
        let Some(&(_, y)) = self.steps.iter().find(|s| s.0 >= n) else { return r.y0 + k as i32 };
        if y > self.since || self.since > 0 {
            return r.y0 + y as i32;
        }
        let age = (r.y0 - self.joined).max(0) as f32;
        let (people, first) = (n as f32 * world::PEOPLE_TO_A_HOUSE, self.steps[0].0 as f32 * world::PEOPLE_TO_A_HOUSE);
        if people <= 300.0 || first <= 300.0 || age <= 0.0 {
            // the cottages that were there when the town took the ground, or went up as it did
            return self.joined - (own * 14.0) as i32;
        }
        let share = ((people - 300.0) / (first - 300.0)).clamp(0.0, 1.0);
        let after = -30.0 * (1.0 - share * (1.0 - (-age / 30.0).exp())).max(1e-6).ln();
        self.joined + after.min(age) as i32
    }
}

/// A house plot on kept ground: a `Plot`, and its turn.
struct Lot {
    pl: Plot,
    /// the cell whose houses it is one of
    cell: u32,
    /// when its turn comes among the plots of its cell: the place of its close in the cell's order, and a little of its own
    turn: f32,
    /// what kind of street it fronts (`Way::kind`; 9 round a market place)
    kind: u8,
    /// its street is a road, or a market place
    road: bool,
}

/// A house that stands this year.
struct Stood {
    lot: Lot,
    /// how many houses its cell had before it, the calendar year it went up, and its roof
    before: usize,
    year: i32,
    roof: Roof,
    neighbours: (bool, bool),
}

/// The roof of a house built in a calendar year on the ground of cell `wc`, as it is in year `k`.
fn roof_in(r: &Replay, k: usize, wc: usize, year: i32, seed: Seed) -> Roof {
    let now = r.y0 + k as i32;
    let laid = (Roof::laid(year, now, seed) - r.y0).clamp(0, k as i32) as usize;
    Roof::of(year, now, r.terrain[wc] >= 2, r.trans[laid * r.n + wc] & 4 != 0, seed)
}

/// The link between two neighbouring cells as the roads number them: the
/// lower cell, and which way the other lies (east, south, south-east, south-west).
fn link(r: &Replay, a: usize, b: usize) -> Option<(usize, usize)> {
    let (lo, hi) = (a.min(b), a.max(b));
    let dir = match hi - lo {
        1 => 0,
        d if d == r.w => 1,
        d if d == r.w + 1 => 2,
        d if d + 1 == r.w => 3,
        _ => return None,
    };
    Some((lo, dir))
}

/// Every street that could be laid this year on kept town ground: the roads
/// that touch it, a lane from each cell to the older cell it grew from where
/// no road runs between them yet (when the road comes it takes the lane's
/// place, and its plots), and lanes and cross streets behind each.
fn ways(r: &Replay, cen: &[(f32, f32)], k: usize, town: &world::TownGround, sq: &[Square]) -> Vec<Way> {
    let on_town = |c: usize| r.built[k * r.n + c] > 0;
    // (lower cell, direction) -> the road's level, or 0 for a lane
    let mut links: std::collections::BTreeMap<(usize, usize), u8> = Default::default();
    for (c, m, lv) in crate::network::road_segments(r, k) {
        if on_town(c) || on_town(m) {
            if let Some(key) = link(r, c, m) {
                links.insert(key, lv);
            }
        }
    }
    let mut cells: Vec<(&u32, &world::TownCell)> = town.cells.iter().collect();
    cells.sort_by_key(|c| *c.0);
    for (&c, cell) in cells {
        if let Some(key) = cell.parent.and_then(|p| link(r, c as usize, p as usize)) {
            links.entry(key).or_insert(0);
        }
    }
    // a town with no road at all still has a street through its market place
    for s in sq {
        let node = r.header.towns[s.town].node;
        let touched = links.keys().any(|&(lo, dir)| lo == node || lo + [1, r.w, r.w + 1, r.w - 1][dir] == node);
        if !touched && on_town(node) && node % r.w + 1 < r.w {
            links.insert((node, 0), 0);
        }
    }
    let mut out = Vec::new();
    let node = |c: usize| r.header.towns.iter().any(|t| t.node == c);
    for (&(c, dir), &lv) in &links {
        let m = c + [1, r.w, r.w + 1, r.w - 1][dir];
        // the lanes behind a way stop short of a market place, and run nearly to the bend where the way only passes through
        let (short_a, short_b) = (if node(c) { 0.1 } else { 0.035 }, if node(m) { 0.1 } else { 0.035 });
        let (a, b) = (v2(cen[c]), v2(cen[m]));
        let sid = ((c * 4 + dir) * 64) as i32;
        out.push(Way { a, b, width: if lv == 3 { 0.034 } else if lv == 2 { 0.022 } else { 0.016 }, id: sid, road: lv >= 2, kind: 0, main: sid });
        let along = b - a;
        let len = along.length().max(1e-4);
        let u = along / len;
        let n = Vec2::new(-u.y, u.x);
        for (j, side) in [(0, -1.0f32), (1, 1.0)] {
            for (ring, off) in [(0, BACK_LANE), (1, OUTER_LANE), (2, FAR_LANE)] {
                let o = n * side * off;
                let id = sid + 2 + ring * 2 + j;
                let jit = 0.06 * (world::hash2(id, 1, 22) - 0.5);
                let jit = jit * if short_a < 0.1 && short_b < 0.1 { 0.4 } else { 1.0 };
                out.push(Way { a: a + u * (short_a + jit.max(-0.03)) + o, b: b - u * (short_b - jit.min(0.03)) + o, width: 0.014, id, road: false, kind: 1 + ring as u8, main: sid });
            }
        }
        let count = ((len - 0.2) / CROSS_EVERY).floor().max(0.0) as i32;
        for i in 0..count {
            let t = (0.1 + CROSS_EVERY * (i as f32 + 0.5)) / len + 0.04 * (world::hash2(sid, i, 23) - 0.5);
            let base = a + along * t.clamp(0.08, 0.92);
            for (j, side) in [(0, -1.0f32), (1, 1.0)] {
                let id = sid + 8 + i * 2 + j;
                let h = world::hash2(id, 3, 23);
                let reach = if h < 0.4 { BACK_LANE } else if h < 0.8 { OUTER_LANE } else { FAR_LANE };
                let w = n * side;
                out.push(Way { a: base + w * 0.02, b: base + w * reach, width: 0.012, id, road: false, kind: 4, main: sid });
            }
        }
    }
    for s in sq {
        if r.trans[k * r.n + r.header.towns[s.town].node] & 4 != 0 {
            let st = station_at(s);
            let d = (st - s.at).normalize_or_zero();
            out.push(Way { a: s.at + d * s.radius, b: st - d * 0.03, width: 0.02, id: -1 - s.town as i32, road: false, kind: 5, main: -1 - s.town as i32 });
        }
    }
    out
}

/// The houses of the kept towns for year `k`, built into `into`.
#[allow(clippy::too_many_arguments)]
pub(super) fn houses_kept(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, lk: &Look, parishes: &[Parish], near_plot: &dyn Fn(f32, f32) -> bool, land: &world::YearLand, into: &mut Into) -> Census {
    let town = &land.town;
    let sq = squares(r, cen, k);
    let t0 = std::time::Instant::now();
    let in_square = |p: Vec2| sq.iter().any(|s| p.distance(s.at) < s.radius + 0.012 || p.distance(station_at(s)) < 0.07) || parishes.iter().any(|c| c.crowded_by(p));
    // the close a spot lies in, if it is built-up ground of a kept town cell: the cell, and the close's place in its order
    let mut closes = world::CloseLookup::new(g);
    let mut built_up = |p: Vec2| -> Option<(u32, u16)> {
        let (parcel, sub) = closes.at(p.x, p.y);
        if !town.built_up(parcel, sub) || world::height_at(g, p.x, p.y) < 0.05 {
            return None;
        }
        town.rank_of(parcel, sub).map(|rank| (g.land.wcell[parcel as usize], rank))
    };
    // the cells of the map with a market place, a station or a church in them or next to them
    let mut by_square: std::collections::HashSet<(i32, i32)> = Default::default();
    for p in sq.iter().flat_map(|s| [s.at, station_at(s)]).chain(parishes.iter().map(|c| c.at)) {
        for (dx, dy) in (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (dx, dy))) {
            by_square.insert((p.x.floor() as i32 + dx, p.y.floor() as i32 + dy));
        }
    }
    let in_square = |p: Vec2| by_square.contains(&(p.x.floor() as i32, p.y.floor() as i32)) && in_square(p);

    // lay the streets: each is walked in short steps and laid where its ground is built up
    let mut laid: Vec<(Way, Vec<(f32, f32)>)> = Vec::new();
    let mut street_pts: HashMap<(i32, i32), Vec<(Vec2, f32)>> = HashMap::new();
    let cell = |p: Vec2| ((p.x / 0.05).floor() as i32, (p.y / 0.05).floor() as i32);
    for s in ways(r, cen, k, town, &sq) {
        let len = s.a.distance(s.b);
        let steps = ((len / 0.03).ceil() as usize).max(1);
        let mut runs: Vec<(f32, f32)> = Vec::new();
        let mut start: Option<f32> = None;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let p = s.a.lerp(s.b, t);
            let ok = built_up(p).is_some() && (s.road || (!near_plot(p.x, p.y) && !in_square(p)));
            match (ok, start) {
                (true, None) => start = Some(t),
                (false, Some(t0)) => {
                    runs.push((t0, (i as f32 - 1.0) / steps as f32));
                    start = None;
                }
                _ => {}
            }
        }
        if let Some(t0) = start {
            runs.push((t0, 1.0));
        }
        runs.retain(|(t0, t1)| (t1 - t0) * len > 0.05);
        for &(t0, t1) in &runs {
            let n = (((t1 - t0) * len / 0.03).ceil() as usize).max(1);
            for i in 0..=n {
                let p = s.a.lerp(s.b, t0 + (t1 - t0) * i as f32 / n as f32);
                street_pts.entry(cell(p)).or_default().push((p, s.width * 0.5));
            }
        }
        if !runs.is_empty() {
            laid.push((s, runs));
        }
    }
    let on_street = |p: Vec2, clear: f32| -> bool {
        let c = cell(p);
        (-1..=1).any(|dy| (-1..=1).any(|dx| street_pts.get(&(c.0 + dx, c.1 + dy)).is_some_and(|v| v.iter().any(|&(q, hw)| q.distance(p) < hw + clear))))
    };

    let t1 = std::time::Instant::now();
    // plots along both sides of every laid street, at fixed places along it, then round each market place
    let mut cands: Vec<Lot> = Vec::new();
    for (s, runs) in &laid {
        if s.kind == 5 {
            continue;
        }
        let along = s.b - s.a;
        let len = along.length().max(1e-4);
        let u = along / len;
        let n = Vec2::new(-u.y, u.x);
        // streets differ: the width of their plots and how far their houses stand back are the street's own
        let (h1, h2) = (world::hash2(s.id, 5, 31), world::hash2(s.id, 6, 31));
        let spacing = match s.kind {
            0 if s.road => 0.048 + 0.012 * h1,
            0 => 0.056 + 0.012 * h1,
            1 => houses::BACK_PLOT_WIDTH,
            3 => 0.056 + 0.01 * h1,
            _ => houses::BACK_PLOT_WIDTH + 0.006 * h1,
        };
        let depth = houses::PLOT.1;
        let back = if s.kind == 0 && !s.road || s.kind == 3 { 0.005 * h2 } else { 0.0 };
        let set_back = s.width * 0.5 + depth * 0.5 + houses::SET_BACK + back;
        let yaw_along = (-u.y).atan2(u.x);
        let count = (len / spacing).floor() as i32;
        let t_of = |i: i32| (i as f32 + 0.5) * spacing / len;
        let laid_at = |t: f32| runs.iter().any(|&(t0, t1)| t >= t0 && t <= t1);
        // a family of colours to a street: most streets keep their houses' second number near one of their own
        let family = (world::hash2(s.id, 7, 31) < 0.7).then(|| 0.16 + 0.68 * world::hash2(s.id, 8, 31));
        for (j, side) in [(0u8, -1.0f32), (1, 1.0)] {
            let id = s.id * 4 + j as i32;
            // the back streets are built a row at a time: rows of three to six plots, fixed per street
            let mut rows: Vec<(i32, i32, i32)> = Vec::new();
            if s.kind != 0 {
                let mut i = 0;
                while i < count {
                    let l = 3 + (world::hash2(id, rows.len() as i32, 28) * 4.0) as i32;
                    rows.push((rows.len() as i32, i, (i + l).min(count)));
                    i += l;
                }
            }
            for i in 0..count {
                let row = rows.iter().find(|&&(_, a, b)| i >= a && i < b).copied();
                if !laid_at(t_of(i)) {
                    continue;
                }
                let p = s.a + along * t_of(i) + n * side * set_back;
                let Some((c, rank)) = built_up(p) else { continue };
                let (pu, pv) = (world::hash2(id, i, 24), world::hash2(id, i, 25));
                let v = family.map_or(pv, |f| (f + 0.3 * (pv - 0.5)).clamp(0.0, 0.999));
                // the frontage of the way itself fills before the lanes behind it
                let own = if s.kind == 0 { 0.6 * pu } else { 0.25 + 0.75 * pu };
                cands.push(Lot {
                    pl: Plot { p, yaw: yaw_along + if side > 0.0 { std::f32::consts::PI } else { 0.0 }, run: (id, j), row: row.map(|r| r.0), high: s.kind == 0, index: i, width: spacing, depth, need: 0.0, u: pu, v },
                    cell: c,
                    turn: rank as f32 + THIN * own,
                    kind: s.kind,
                    road: s.road,
                });
            }
        }
    }
    for s in &sq {
        let count = ((std::f32::consts::TAU * (s.radius + 0.03)) / 0.05).floor() as i32;
        for i in 0..count {
            let a = i as f32 / count as f32 * std::f32::consts::TAU;
            let dirv = Vec2::new(a.cos(), a.sin());
            let p = s.at + dirv * (s.radius + 0.03);
            let id = -1000 - s.town as i32;
            let pu = world::hash2(id, i, 26);
            let Some((c, _)) = built_up(p) else { continue };
            // the market place is the first of a town to be built round
            cands.push(Lot {
                pl: Plot { p, yaw: (dirv.y).atan2(-dirv.x) + std::f32::consts::FRAC_PI_2, run: (id, 0), row: None, high: true, index: i, width: 0.05, depth: houses::PLOT.1, need: 0.0, u: pu, v: world::hash2(id, i, 27) },
                cell: c,
                turn: -2.0 + pu,
                kind: 9,
                road: true,
            });
        }
    }

    let t2 = std::time::Instant::now();
    // a row's turn is its first plot's, so that it goes up together
    let mut row_turn: HashMap<(i32, i32), f32> = HashMap::new();
    for l in &cands {
        if let Some(ri) = l.pl.row {
            let t = row_turn.entry((l.pl.run.0, ri)).or_insert(f32::MAX);
            *t = t.min(l.turn);
        }
    }
    for l in &mut cands {
        if let Some(ri) = l.pl.row {
            l.turn = row_turn[&(l.pl.run.0, ri)];
        }
    }
    // in the order their turns come: the plots that can be built keep their ground against later ones
    cands.sort_by(|a, b| a.turn.total_cmp(&b.turn).then(a.pl.run.cmp(&b.pl.run)).then(a.pl.index.cmp(&b.pl.index)));
    let mut taken: HashMap<(i32, i32), Vec<Vec2>> = HashMap::new();
    let mut stories: HashMap<u32, (Story, usize)> = HashMap::new();
    let mut standing: Vec<Stood> = Vec::new();
    // how many houses a row brought its cell, so that the whole of it comes in or none
    let mut row_in: HashMap<(i32, i32), bool> = HashMap::new();
    for lot in cands {
        let p = lot.pl.p;
        // has its cell houses left to build? (asked first: it is the cheapest to answer)
        let (story, built) = stories.entry(lot.cell).or_insert_with(|| (Story::of(r, k, lot.cell as usize), 0));
        let room = *built < story.houses;
        match lot.pl.row.map(|ri| row_in.get(&(lot.pl.run.0, ri)).copied()) {
            Some(Some(false)) => continue,
            Some(Some(true)) => {}
            _ if !room => {
                if let Some(ri) = lot.pl.row {
                    row_in.insert((lot.pl.run.0, ri), false);
                }
                continue;
            }
            _ => {}
        }
        if near_plot(p.x, p.y) || in_square(p) || on_street(p, if lot.kind == 9 { 0.03 } else { 0.02 }) {
            continue;
        }
        let c = cell(p);
        if (-1..=1).any(|dy| (-1..=1).any(|dx| taken.get(&(c.0 + dx, c.1 + dy)).is_some_and(|v| v.iter().any(|q| q.distance(p) < 0.042)))) {
            continue;
        }
        if let Some(ri) = lot.pl.row {
            row_in.insert((lot.pl.run.0, ri), true);
        }
        taken.entry(c).or_default().push(p);
        let seed = Seed::new(lot.pl.u, lot.pl.v);
        // a row is of the year its first house's turn came
        let before = *built;
        *built += 1;
        let year = story.year_of(r, k, before + 1, lot.pl.u);
        let roof = roof_in(r, k, lot.cell as usize, year, seed);
        standing.push(Stood { lot, before, year, roof, neighbours: (false, false) });
    }
    // a row shares the year and the place in the count of its first house
    let mut row_first: HashMap<(i32, i32), (usize, i32)> = HashMap::new();
    for s in &standing {
        if let Some(ri) = s.lot.pl.row {
            let e = row_first.entry((s.lot.pl.run.0, ri)).or_insert((s.before, s.year));
            if s.before < e.0 {
                *e = (s.before, s.year);
            }
        }
    }
    for s in &mut standing {
        if let Some(ri) = s.lot.pl.row {
            (s.before, s.year) = row_first[&(s.lot.pl.run.0, ri)];
        }
    }

    let t3 = std::time::Instant::now();
    // the lanes, where houses front them: a lane is drawn along the houses it has, a
    // cross street from the way it leaves to its last house (or to the lane it reaches,
    // if that has houses there), and the way out of an older cell wherever it is laid
    let way_at: HashMap<i32, usize> = laid.iter().enumerate().map(|(i, (w, _))| (w.id, i)).collect();
    let mut fronted: HashMap<i32, (f32, f32)> = HashMap::new();
    for s in &standing {
        if s.lot.kind == 9 {
            continue;
        }
        let way = s.lot.pl.run.0.div_euclid(4);
        if let Some((w, _)) = way_at.get(&way).map(|&i| &laid[i]) {
            let t = (s.lot.pl.p - w.a).dot(w.b - w.a) / w.a.distance_squared(w.b).max(1e-9);
            let e = fronted.entry(way).or_insert((t, t));
            *e = (e.0.min(t), e.1.max(t));
        }
    }
    let colour = *lk.ground.street_earth;
    for (s, runs) in &laid {
        if s.road {
            continue;
        }
        let len = s.a.distance(s.b).max(1e-4);
        let half = 0.03 / len;
        let spans: Vec<(f32, f32)> = match s.kind {
            0 | 5 => runs.clone(),
            4 => {
                // the lane it runs out to, and whether houses stand on that lane where it arrives
                let ring = if len > OUTER_LANE + 0.05 { 2 } else if len > BACK_LANE + 0.05 { 1 } else { 0 };
                let lane = s.main + 2 + ring * 2 + (s.id - s.main - 8).rem_euclid(2);
                let arrives = way_at.get(&lane).map(|&i| &laid[i]).zip(fronted.get(&lane)).is_some_and(|((w, _), &(t0, t1))| {
                    let t = (s.b - w.a).dot(w.b - w.a) / w.a.distance_squared(w.b).max(1e-9);
                    let slack = 0.1 / w.a.distance(w.b).max(1e-4);
                    t >= t0 - slack && t <= t1 + slack
                });
                match (arrives, fronted.get(&s.id)) {
                    (true, _) => vec![(0.0, 1.0)],
                    (false, Some(&(_, t1))) => vec![(0.0, (t1 + half).min(1.0))],
                    _ => Vec::new(),
                }
            }
            _ => fronted.get(&s.id).map(|&(t0, t1)| vec![((t0 - half).max(0.0), (t1 + half).min(1.0))]).unwrap_or_default(),
        };
        for (t0, t1) in spans {
            // only where the street is laid
            for &(r0, r1) in runs {
                let (a, b) = (t0.max(r0), t1.min(r1));
                if (b - a) * len < 0.03 {
                    continue;
                }
                let n = (((b - a) * len / 0.03).ceil() as usize).max(1);
                let pts: Vec<Vec3> = (0..=n)
                    .map(|i| {
                        let p = s.a.lerp(s.b, a + (b - a) * i as f32 / n as f32);
                        Vec3::new(p.x, world::height_at(g, p.x, p.y).max(0.0) + 0.006, p.y)
                    })
                    .collect();
                into.houses.ribbon(&pts, s.width, colour);
            }
        }
    }

    // the houses, street by street
    standing.sort_by_key(|s| (s.lot.pl.run, s.lot.pl.index));
    for i in 0..standing.len() {
        let next_to = |j: usize| standing[j].lot.pl.run == standing[i].lot.pl.run && (standing[j].lot.pl.index - standing[i].lot.pl.index).abs() == 1;
        let (before, after) = (i > 0 && next_to(i - 1), i + 1 < standing.len() && next_to(i + 1));
        standing[i].neighbours = if standing[i].lot.pl.run.1 == 1 { (after, before) } else { (before, after) };
    }
    let mut census = Census::default();
    for (c, (story, _)) in &stories {
        let _ = c;
        census.wanted += story.houses;
    }
    // cells with no plot at all still count
    for (&c, cell) in &town.cells {
        census.cells += 1;
        census.residents += r.residents(k, c as usize);
        if !stories.contains_key(&c) {
            census.wanted += cell.houses as usize;
        }
    }
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    let mut i = 0;
    while i < standing.len() {
        let first = &standing[i];
        let story = &stories[&first.lot.cell].0;
        let mut j = i + 1;
        if first.lot.pl.row.is_some() && first.year >= TERRACE_ERA {
            while j < standing.len() && standing[j].lot.pl.run == first.lot.pl.run && standing[j].lot.pl.row == first.lot.pl.row && standing[j].lot.pl.index == standing[j - 1].lot.pl.index + 1 {
                j += 1;
            }
        }
        if j - i >= 2 {
            // a brick terrace: two storeys, and a third only in a cell that has filled long since
            let (a, b) = (first.lot.pl.p, standing[j - 1].lot.pl.p);
            let mid = (a + b) * 0.5;
            let seed = Seed::new(first.lot.pl.u, first.lot.pl.v);
            let storeys = 2 + (story.houses >= first.before + 120 + (40.0 * seed.u) as usize) as u8;
            let t = Terrace {
                houses: j - i,
                length: a.distance(b) + first.lot.pl.width * 0.96,
                depth: first.lot.pl.depth,
                storeys,
                roof: first.roof,
                era: Era::of(first.year),
                upland: r.terrain[first.lot.cell as usize] >= 2,
                detail: into.lod.at(mid.x, mid.y),
                seed,
            };
            houses::terrace(into.houses, into.smoke, &Site::on(ground(g, mid.x, mid.y), first.lot.pl.yaw, &height), &t, lk);
            census.terraced += j - i;
            census.storeys[(storeys as usize - 1).min(2)] += j - i;
        } else {
            for s in &standing[i..j] {
                one_house(into, r, k, lk, g, &height, s, &stories[&s.lot.cell].0, sq.iter().any(|q| q.big && s.lot.kind == 9 && q.at.distance(s.lot.pl.p) < 0.2), &mut census);
            }
        }
        i = j;
    }
    census.drawn = standing.len();
    if std::env::var_os("ISOSTATE_TIMING").is_some() {
        let ms = |a: std::time::Instant, b: std::time::Instant| (b - a).as_secs_f64() * 1000.0;
        eprintln!("timing kept towns: streets {:.0} ms, plots {:.0} ms, turns {:.0} ms, houses {:.0} ms", ms(t0, t1), ms(t1, t2), ms(t2, t3), ms(t3, std::time::Instant::now()));
    }
    census
}

/// A house on its own plot: a cottage, a villa, or on a road through the town
/// or its market place, once the cell has grown past it, a townhouse.
#[allow(clippy::too_many_arguments)]
fn one_house(into: &mut Into, r: &Replay, k: usize, lk: &Look, g: &Ground, height: &dyn Fn(f32, f32) -> f32, s: &Stood, story: &Story, great: bool, census: &mut Census) {
    let pl = &s.lot.pl;
    let seed = Seed::new(pl.u, pl.v);
    let upland = r.terrain[s.lot.cell as usize] >= 2;
    let detail = into.lod.at(pl.p.x, pl.p.y);
    let site = Site::on(ground(g, pl.p.x, pl.p.y), pl.yaw, height);
    let (joined, both) = (s.neighbours.0 || s.neighbours.1, s.neighbours.0 && s.neighbours.1);
    // villas: on the lanes and the ways out, on ground with little works on it, and never in a row
    let villa = pl.row.is_none() && s.lot.kind != 9 && !story.node && story.works < 0.15 && houses::villa_plot(s.year, seed);
    // rebuilt as a townhouse, the full width of its plot, once the cell has this many houses:
    // soon on a road through the town or round its market place, later on the old back
    // streets (those of the brick years are terraces from the first)
    let rebuilt = s.before + if s.lot.road { 11 + (40.0 * pl.v) as usize } else { 30 + (60.0 * pl.v) as usize };
    if villa {
        let v = Villa { plot_width: pl.width, plot_depth: pl.depth, roof: s.roof, era: Era::of(s.year), upland, detail, seed };
        houses::villa(into.houses, into.smoke, &mut into.trees[leaf_group(pl.v)], &site, &v, lk);
        census.villas += 1;
        census.storeys[1] += 1;
    } else if joined && story.houses >= rebuilt && (s.lot.road || s.year < TERRACE_ERA) {
        // a third storey on a road where the frontage is whole and the cell has grown as much
        // again; a fourth only round the market place of a great town
        let third = s.lot.road && both && story.houses >= rebuilt + 45 + (70.0 * pl.u) as usize;
        let fourth = third && great && story.houses >= rebuilt + 150;
        let storeys = 2 + third as u8 + fourth as u8;
        let t = Townhouse {
            plot_width: pl.width,
            plot_depth: pl.depth,
            storeys,
            roof: s.roof,
            era: Era::of(story.year_of(r, k, rebuilt, pl.u).max(s.year)),
            upland,
            shop: s.lot.road && houses::shop_at(((story.houses - s.before) as f32 / 90.0).min(1.0), seed),
            neighbours: s.neighbours,
            detail,
            seed,
        };
        houses::townhouse(into.houses, into.smoke, &site, &t, lk);
        census.townhouses += 1;
        census.storeys[(storeys as usize - 1).min(2)] += 1;
        census.tall_alone += (storeys >= 3 && !both) as usize;
    } else {
        // a cottage: one storey, and a second once the cell has grown well past it
        let storeys = 1 + (story.houses >= s.before + 45 + (110.0 * pl.v) as usize) as u8;
        let c = Cottage { plot_width: pl.width, plot_depth: pl.depth, storeys, roof: s.roof, era: Era::of(s.year), upland, neighbours: s.neighbours, detail, seed };
        houses::cottage(into.houses, into.smoke, &mut into.trees[leaf_group(pl.v)], &site, &c, lk);
        census.cottages += 1;
        census.storeys[storeys as usize - 1] += 1;
    }
}
