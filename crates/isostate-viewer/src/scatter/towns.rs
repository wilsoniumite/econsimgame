//! Towns, laid out the way market towns grew. There is a market place with the
//! church on one side. High streets run along the roads in, with an unbroken
//! frontage near the middle. Back lanes run behind the plots, with cross streets
//! between them, and as a town fills up its streets turn into terraced rows. A
//! station stands at the end of its own road. Every street and plot is fixed by
//! where the roads run and appears once its ground is built up enough, so a town
//! thickens in place from year to year.

use super::*;
use bevy::math::Vec2;
use std::collections::HashMap;

/// Packed earth and setts for the back streets, and stone for the market place.
const STREET: u32 = 0xc4ad87;
const SQUARE: u32 = 0xd8cdb6;
/// Brick for terraces, from fresh red to sooty brown.
const BRICK: [u32; 4] = [0xb5634c, 0xa65a45, 0xc07a5a, 0x96574a];

/// Where a road enters town and what lies along it, in world units.
const BACK_LANE: f32 = 0.13;
const OUTER_LANE: f32 = 0.26;
const CROSS_EVERY: f32 = 0.17;

/// A street that may be laid this year: a straight run with the density its
/// ground needs first. Main streets are the roads themselves, already drawn.
struct Street {
    a: Vec2,
    b: Vec2,
    width: f32,
    need: f32,
    id: i32,
    road: bool,
}

/// A market place: where it is, how big, and which way the town faces.
struct Square {
    at: Vec2,
    radius: f32,
    yaw: f32,
    town: usize,
    big: bool,
}

/// A house plot fronting a street.
struct Plot {
    p: Vec2,
    yaw: f32,
    /// the street and side it fronts and its place along it, for terraces
    run: (i32, u8),
    /// it fronts a road through town: the old high street rather than a newer lane
    high: bool,
    index: i32,
    width: f32,
    depth: f32,
    need: f32,
    u: f32,
    v: f32,
}

fn v2((x, z): (f32, f32)) -> Vec2 {
    Vec2::new(x, z)
}

/// The square each active town has, sized by its population.
fn squares(r: &Replay, cen: &[(f32, f32)], k: usize) -> Vec<Square> {
    let mut out = Vec::new();
    for t in 0..r.n_towns.min(r.header.towns.len()) {
        if r.town_active[k * r.n_towns + t] == 0 {
            continue;
        }
        let urban = r.town_urban[k * r.n_towns + t];
        out.push(Square {
            at: v2(cen[r.header.towns[t].node]),
            radius: 0.055 + 0.035 * (urban / 20000.0).clamp(0.0, 1.0),
            yaw: (world::hash2(t as i32, 1, 5) * 4.0).floor() * std::f32::consts::FRAC_PI_2,
            town: t,
            big: urban > 12000.0,
        });
    }
    out
}

/// Where a town's market square is in a year: its centre, radius and the way it faces.
pub fn square_of(r: &Replay, cen: &[(f32, f32)], k: usize, t: usize) -> Option<(Vec2, f32, f32)> {
    squares(r, cen, k).into_iter().find(|s| s.town == t).map(|s| (s.at, s.radius, s.yaw))
}

/// The church faces the square from its east side, the station is down its own road to the south.
fn church_at(s: &Square) -> Vec2 {
    let q = bevy::math::Quat::from_rotation_y(s.yaw);
    let o = q * Vec3::new(s.radius + 0.05, 0.0, 0.0);
    s.at + Vec2::new(o.x, o.z)
}

fn station_at(s: &Square) -> Vec2 {
    let q = bevy::math::Quat::from_rotation_y(s.yaw);
    let o = q * Vec3::new(0.0, 0.0, 0.24);
    s.at + Vec2::new(o.x, o.z)
}

pub(super) fn landmarks(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, houses: &mut Builder, plots: &mut HashMap<usize, Vec<(f32, f32)>>) {
    for s in squares(r, cen, k) {
        let node = r.header.towns[s.town].node;
        // the market place, paved, with a cross or a hall in the middle
        let paving = hex(SQUARE);
        let n: usize = 9;
        let mut ring = Vec::new();
        for i in 0..n {
            let a = i as f32 / n as f32 * std::f32::consts::TAU + world::hash2(s.town as i32, i as i32, 6) * 0.3;
            let rr = s.radius * (0.85 + 0.3 * world::hash2(s.town as i32, i as i32, 7));
            let p = s.at + Vec2::new(a.cos(), a.sin()) * rr;
            ring.push(Vec3::new(p.x, world::height_at(g, p.x, p.y) + 0.009, p.y));
        }
        let mid = Vec3::new(s.at.x, world::height_at(g, s.at.x, s.at.y) + 0.009, s.at.y);
        for i in 0..n {
            houses.tri(mid, ring[(i + 1) % n], ring[i], paving);
        }
        let m = ground(g, s.at.x, s.at.y);
        if s.big {
            // a market hall on stone pillars
            let stone = hex(0xddd5c4);
            for (dx, dz) in [(-0.016, -0.01), (0.016, -0.01), (-0.016, 0.01), (0.016, 0.01)] {
                let q = bevy::math::Quat::from_rotation_y(s.yaw);
                let o = q * Vec3::new(dx, 0.0, dz);
                houses.beam(m + o, m + o + Vec3::Y * 0.014, 0.004, stone);
            }
            houses.block(m + Vec3::Y * 0.014, 0.044, 0.03, 0.012, s.yaw, hex(0xf0dcb8), None, 0.0);
            houses.gable(m + Vec3::Y * 0.014, 0.044, 0.03, 0.012, 0.014, s.yaw, 0.003, hex(0xb5523f), hex(0xf0dcb8));
        } else {
            // a market cross on its steps
            let stone = hex(0xd9d0be);
            houses.block(m, 0.016, 0.016, 0.004, s.yaw, stone, Some(stone), 0.0);
            houses.beam(m, m + Vec3::Y * 0.03, 0.0035, stone);
            houses.beam(m + Vec3::new(-0.006, 0.024, 0.0), m + Vec3::new(0.006, 0.024, 0.0), 0.003, stone);
        }
        let c = church_at(&s);
        church(houses, ground(g, c.x, c.y), s.yaw + std::f32::consts::FRAC_PI_2, s.big);
        plots.entry(node).or_default().push((c.x, c.y));
        plots.entry(node).or_default().push((s.at.x, s.at.y));
        if r.trans[k * r.n + node] & 4 != 0 {
            let st = station_at(&s);
            station(houses, ground(g, st.x, st.y), s.yaw);
            plots.entry(node).or_default().push((st.x, st.y));
        }
        // a lighthouse if the town is a harbour: walk toward the sea to the waterline
        let (cx, cy) = ((node % r.w) as isize, (node / r.w) as isize);
        let mut sea: Option<(f32, f32)> = None;
        for (dx, dy) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)] {
            let (nx, ny) = (cx + dx, cy + dy);
            if nx >= 0 && ny >= 0 && (nx as usize) < r.w && (ny as usize) < r.h && r.terrain[ny as usize * r.w + nx as usize] == 0 {
                sea = Some((dx as f32, dy as f32));
                break;
            }
        }
        if let Some((dx, dy)) = sea {
            let (mut lx, mut lz) = (s.at.x, s.at.y);
            for _ in 0..40 {
                let (nx, nz) = (lx + dx * 0.025, lz + dy * 0.025);
                if world::height_at(g, nx, nz) < 0.03 {
                    break;
                }
                lx = nx;
                lz = nz;
            }
            lighthouse(houses, ground(g, lx, lz));
        }
    }
}

/// The density a point's ground is built up to: a town square's own reading, or a guess for
/// town patches in mixed squares.
fn density(r: &Replay, k: usize, c: usize) -> f32 {
    let off = k * r.n + c;
    if r.use_[off] == 10 { r.inten[off] as f32 / 255.0 } else { 0.4 }
}

/// The class of the ground at a point, by the same patchwork rule the ground is painted with.
fn class_here(r: &Replay, g: &Ground, k: usize, x: f32, z: f32) -> (u8, usize) {
    let (wx, wy) = world::warp(x, z);
    let wc = (wy.floor().clamp(0.0, r.h as f32 - 1.0) as usize) * r.w + wx.floor().clamp(0.0, r.w as f32 - 1.0) as usize;
    (world::pick_class(r, k, wc, world::sub_uniform(g, x, z) * 255.0), wc)
}

/// Every street that could be laid this year: each road through town ground, and
/// back lanes, outer lanes and cross streets laid out along it.
fn streets(r: &Replay, cen: &[(f32, f32)], k: usize, sq: &[Square]) -> Vec<Street> {
    let townish = |c: usize| {
        let off = k * r.n + c;
        r.use_[off] == 10 || r.mix_a[off] == world::URBAN || r.mix_b[off] == world::URBAN
    };
    let mut out = Vec::new();
    for (c, m, lv) in crate::network::road_segments(r, k) {
        if !townish(c) && !townish(m) {
            continue;
        }
        let (a, b) = (v2(cen[c]), v2(cen[m]));
        let dir = if m == c + 1 { 0 } else if m == c + r.w { 1 } else if m == c + r.w + 1 { 2 } else { 3 };
        let sid = ((c * 4 + dir) * 64) as i32;
        out.push(Street { a, b, width: if lv == 3 { 0.034 } else { 0.022 }, need: 0.0, id: sid, road: true });
        let along = b - a;
        let len = along.length().max(1e-4);
        let u = along / len;
        let n = Vec2::new(-u.y, u.x);
        // lanes behind the frontage plots on both sides, and a second ring further out
        for (j, side) in [(0, -1.0f32), (1, 1.0)] {
            for (ring, off, need) in [(0, BACK_LANE, 0.3), (1, OUTER_LANE, 0.62)] {
                let o = n * side * off;
                let id = sid + 2 + ring * 2 + j;
                let jit = 0.06 * (world::hash2(id, 1, 22) - 0.5);
                out.push(Street { a: a + u * (0.1 + jit) + o, b: b - u * (0.1 - jit) + o, width: 0.014, need: need + 0.08 * world::hash2(id, 2, 22), id, road: false });
            }
        }
        // cross streets from the high street out to the lanes
        let count = ((len - 0.2) / CROSS_EVERY).floor().max(0.0) as i32;
        for i in 0..count {
            let t = (0.1 + CROSS_EVERY * (i as f32 + 0.5)) / len + 0.04 * (world::hash2(sid, i, 23) - 0.5);
            let base = a + along * t.clamp(0.08, 0.92);
            for (j, side) in [(0, -1.0f32), (1, 1.0)] {
                let id = sid + 8 + i * 2 + j;
                let reach = if world::hash2(id, 3, 23) < 0.5 { OUTER_LANE } else { BACK_LANE };
                let w = n * side;
                out.push(Street { a: base + w * 0.02, b: base + w * reach, width: 0.012, need: 0.42 + 0.1 * world::hash2(id, 4, 23), id, road: false });
            }
        }
    }
    // a road from each market place down to its station
    for s in sq {
        if r.trans[k * r.n + r.header.towns[s.town].node] & 4 != 0 {
            let st = station_at(s);
            let d = (st - s.at).normalize_or_zero();
            out.push(Street { a: s.at + d * s.radius, b: st - d * 0.03, width: 0.02, need: 0.0, id: -1 - s.town as i32, road: false });
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
pub(super) fn houses(r: &Replay, g: &Ground, slots: &Slots, cen: &[(f32, f32)], k: usize, near_plot: &dyn Fn(f32, f32) -> bool, houses: &mut Builder, smoke: &mut Builder) {
    let sq = squares(r, cen, k);
    let in_square = |p: Vec2| sq.iter().any(|s| p.distance(s.at) < s.radius + 0.012 || p.distance(church_at(s)) < 0.07 || p.distance(station_at(s)) < 0.07);
    let built = |p: Vec2, need: f32| -> bool {
        if world::height_at(g, p.x, p.y) < 0.05 {
            return false;
        }
        let (cls, wc) = class_here(r, g, k, p.x, p.y);
        (cls == world::URBAN || cls == world::WORKS) && density(r, k, wc) >= need
    };

    // lay the streets: each candidate is walked in short steps and drawn where its ground is built up
    let mut laid: Vec<(Street, Vec<(f32, f32)>)> = Vec::new();
    let mut street_pts: HashMap<(i32, i32), Vec<(Vec2, f32)>> = HashMap::new();
    let cell = |p: Vec2| ((p.x / 0.05).floor() as i32, (p.y / 0.05).floor() as i32);
    let colour = hex(STREET);
    for s in streets(r, cen, k, &sq) {
        let len = s.a.distance(s.b);
        let steps = ((len / 0.03).ceil() as usize).max(1);
        let mut runs: Vec<(f32, f32)> = Vec::new();
        let mut start: Option<f32> = None;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let p = s.a.lerp(s.b, t);
            let ok = if s.road { built(p, 0.0) } else { built(p, s.need) && !near_plot(p.x, p.y) && !in_square(p) };
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
            let pts: Vec<Vec3> = (0..=n)
                .map(|i| {
                    let p = s.a.lerp(s.b, t0 + (t1 - t0) * i as f32 / n as f32);
                    street_pts.entry(cell(p)).or_default().push((p, s.width * 0.5));
                    Vec3::new(p.x, world::height_at(g, p.x, p.y).max(0.0) + 0.006, p.y)
                })
                .collect();
            if !s.road {
                houses.ribbon(&pts, s.width, colour);
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

    // plots along both sides of every laid street, then round each market place
    let mut cands: Vec<Plot> = Vec::new();
    for (s, runs) in &laid {
        let along = s.b - s.a;
        let len = along.length().max(1e-4);
        let u = along / len;
        let n = Vec2::new(-u.y, u.x);
        let spacing = if s.road { 0.05 } else { 0.046 };
        let depth = 0.036;
        let set_back = s.width * 0.5 + depth * 0.5 + 0.005;
        let yaw_along = (-u.y).atan2(u.x);
        for &(t0, t1) in runs {
            let count = (((t1 - t0) * len) / spacing).floor() as i32;
            for i in 0..count {
                let t = t0 + (i as f32 + 0.5) * spacing / len;
                for (j, side) in [(0u8, -1.0f32), (1, 1.0)] {
                    let id = s.id * 4 + j as i32;
                    let (pu, pv) = (world::hash2(id, i, 24), world::hash2(id, i, 25));
                    let p = s.a + along * t + n * side * set_back;
                    // a plot comes in once its ground is built up beyond the street's own need
                    let need = s.need + (1.0 - s.need) * pu * if s.road { 0.75 } else { 0.95 };
                    cands.push(Plot {
                        p,
                        yaw: yaw_along + if side < 0.0 { std::f32::consts::PI } else { 0.0 },
                        run: (id, j),
                        high: s.road,
                        index: i,
                        width: spacing,
                        depth,
                        need,
                        u: pu,
                        v: pv,
                    });
                }
            }
        }
    }
    for s in &sq {
        // houses round the market place, facing in
        let count = ((std::f32::consts::TAU * (s.radius + 0.03)) / 0.05).floor() as i32;
        for i in 0..count {
            let a = i as f32 / count as f32 * std::f32::consts::TAU;
            let dirv = Vec2::new(a.cos(), a.sin());
            let p = s.at + dirv * (s.radius + 0.03);
            let id = -1000 - s.town as i32;
            let pu = world::hash2(id, i, 26);
            cands.push(Plot { p, yaw: (dirv.y).atan2(-dirv.x) + std::f32::consts::FRAC_PI_2, run: (id, 0), high: true, index: i, width: 0.05, depth: 0.036, need: 0.55 * pu, u: pu, v: world::hash2(id, i, 27) });
        }
    }

    // the plots built up enough, earliest first: they keep their ground as the town grows
    cands.sort_by(|a, b| a.need.partial_cmp(&b.need).unwrap_or(std::cmp::Ordering::Equal));
    let mut taken: HashMap<(i32, i32), Vec<Vec2>> = HashMap::new();
    let mut standing: Vec<(Plot, f32)> = Vec::new();
    for pl in cands {
        let (cls, wc) = class_here(r, g, k, pl.p.x, pl.p.y);
        if cls != world::URBAN || world::height_at(g, pl.p.x, pl.p.y) < 0.05 {
            continue;
        }
        let dens = density(r, k, wc);
        if dens < pl.need || near_plot(pl.p.x, pl.p.y) || in_square(pl.p) || on_street(pl.p, 0.012) {
            continue;
        }
        let c = cell(pl.p);
        if (-1..=1).any(|dy| (-1..=1).any(|dx| taken.get(&(c.0 + dx, c.1 + dy)).is_some_and(|v| v.iter().any(|q| q.distance(pl.p) < 0.042)))) {
            continue;
        }
        taken.entry(c).or_default().push(pl.p);
        standing.push((pl, dens));
    }

    // neighbouring plots in a built-up street become one terraced row; elsewhere houses stand apart
    let year = r.y0 + k as i32;
    standing.sort_by_key(|(p, _)| (p.run, p.index));
    let mut i = 0;
    while i < standing.len() {
        let (first, dens) = (&standing[i].0, standing[i].1);
        let mut j = i + 1;
        if dens > 0.6 {
            while j < standing.len() && j - i < 6 && standing[j].0.run == first.run && standing[j].0.index == standing[j - 1].0.index + 1 {
                j += 1;
            }
        }
        if j - i >= 3 {
            // the high streets and the market place keep a frontage of separate houses
            // of all heights and colours; the newer streets of the industrial years are
            // laid out in uniform brick terraces
            if first.high || year < 1815 {
                for (pl, d) in &standing[i..j] {
                    townhouse(houses, smoke, g, pl, *d);
                }
            } else {
                terrace(houses, smoke, g, &standing[i..j]);
            }
        } else {
            for (pl, d) in &standing[i..j] {
                cottage(houses, smoke, g, pl, *d);
            }
        }
        i = j;
    }

    // town patches no street reaches get a few cottages of their own
    for s in &slots.houses {
        let c = s.wcell as usize;
        if world::pick_class(r, k, c, s.sub) != world::URBAN || near_plot(s.x, s.z) || s.u > 0.1 + 0.35 * density(r, k, c) {
            continue;
        }
        let p = Vec2::new(s.x, s.z);
        if in_square(p) || on_street(p, 0.2) {
            continue;
        }
        let pl = Plot { p, yaw: (s.yaw / std::f32::consts::FRAC_PI_2).round() * std::f32::consts::FRAC_PI_2, run: (0, 0), high: false, index: 0, width: 0.05, depth: 0.036, need: 0.0, u: s.u, v: s.v };
        cottage(houses, smoke, g, &pl, density(r, k, c));
    }
}

/// A house standing on its own plot, a little narrower than the plot so the gaps show.
fn cottage(houses: &mut Builder, smoke: &mut Builder, g: &Ground, pl: &Plot, dens: f32) {
    let p = ground(g, pl.p.x, pl.p.y);
    let storeys = 1.0 + ((dens > 0.45 && pl.v > 0.5) as u8 as f32) + ((dens > 0.85 && pl.v > 0.85) as u8 as f32);
    let w = pl.width * (0.68 + 0.22 * pl.v);
    let d = pl.depth * (0.9 + 0.2 * pl.u);
    let yaw = pl.yaw + (pl.v - 0.5) * 0.08;
    let wall = hex(WALLS[(pl.v * 6.0) as usize % 6]);
    let roof = hex(ROOFS[(pl.u * 13.0) as usize % 5]);
    house(houses, p, w, d, storeys, yaw, wall, roof);
    windows(houses, p, w, d, storeys, yaw, 1, true);
    if pl.u < 0.3 {
        let cp = p + bevy::math::Quat::from_rotation_y(yaw) * Vec3::new(w * 0.3, 0.0, 0.0);
        if pl.v > 0.8 {
            smoking_chimney(houses, smoke, cp, 0.024 * storeys + 0.03, 0.004, pl.u * 7.0);
        } else {
            chimney(houses, cp, 0.024 * storeys + 0.03, 0.004);
        }
    }
}

/// A terraced row: one long block and roof over several plots, in brick, with a
/// chimney stack on every party wall.
fn terrace(houses: &mut Builder, smoke: &mut Builder, g: &Ground, row: &[(Plot, f32)]) {
    let (a, b) = (row[0].0.p, row[row.len() - 1].0.p);
    let mid = (a + b) * 0.5;
    let unit = row[0].0.width;
    let len = a.distance(b) + unit * 0.96;
    let first = &row[0].0;
    let dens = row.iter().map(|(_, d)| *d).sum::<f32>() / row.len() as f32;
    let storeys = if dens > 0.8 && first.v > 0.4 { 3.0 } else { 2.0 };
    // ground under the row: the lower end, so it never floats
    let h = row.iter().map(|(pl, _)| world::height_at(g, pl.p.x, pl.p.y)).fold(f32::MAX, f32::min);
    let p = Vec3::new(mid.x, h, mid.y);
    let yaw = first.yaw;
    let wall = hex(BRICK[(first.v * 4.0) as usize % 4]);
    let roof = hex(if first.u < 0.75 { 0x55617a } else { 0xb5523f });
    // terraces have lower roofs than cottages
    let h = 0.024 * storeys;
    houses.block(p, len, first.depth, h, yaw, wall, None, 0.03);
    houses.gable(p, len, first.depth, h, first.depth * 0.36, yaw, 0.004, roof, wall);
    windows(houses, p, len, first.depth, storeys, yaw, row.len(), true);
    let q = bevy::math::Quat::from_rotation_y(yaw);
    let top = h + first.depth * 0.36 * 0.55;
    for i in 0..=row.len() {
        let x = -len * 0.5 + len * i as f32 / row.len() as f32;
        let cp = p + q * Vec3::new(x.clamp(-len * 0.5 + 0.004, len * 0.5 - 0.004), 0.0, 0.0);
        if i == row.len() / 2 && first.u < 0.25 {
            smoking_chimney(houses, smoke, cp, top + 0.012, 0.0045, first.v * 9.0);
        } else {
            houses.block(cp + Vec3::Y * top, 0.008, 0.006, 0.01, yaw, hex(0x8f4b3b), Some(hex(0x5a4036)), 0.0);
        }
    }
}

/// Windows and doors on both long faces: two windows a storey for each house in
/// the block, and a door in place of one on the ground floor.
fn windows(b: &mut Builder, p: Vec3, w: f32, d: f32, storeys: f32, yaw: f32, units: usize, door: bool) {
    let q = bevy::math::Quat::from_rotation_y(yaw);
    let glass = hex(0x414552);
    let wood = hex(0x6a4632);
    let unit = w / units as f32;
    let (ww, wh) = ((unit * 0.16).min(0.007), 0.0075);
    for side in [-1.0f32, 1.0] {
        let z = side * (d * 0.5 + 0.0006);
        for u in 0..units {
            let cx = -w * 0.5 + unit * (u as f32 + 0.5);
            for s in 0..(storeys as usize) {
                let y = 0.024 * s as f32 + 0.0125;
                for (k, dx) in [-unit * 0.24, unit * 0.24].into_iter().enumerate() {
                    let (x0, x1, y0, y1, col) = if s == 0 && door && k == 1 && side > 0.0 {
                        (cx + dx - ww * 0.5, cx + dx + ww * 0.5, 0.0005, 0.0145, wood)
                    } else {
                        (cx + dx - ww * 0.5, cx + dx + ww * 0.5, y - wh * 0.5, y + wh * 0.5, glass)
                    };
                    let c = [Vec3::new(x0, y0, z), Vec3::new(x1, y0, z), Vec3::new(x1, y1, z), Vec3::new(x0, y1, z)].map(|v| p + q * v);
                    if side > 0.0 {
                        b.quad(c[0], c[1], c[2], c[3], col);
                    } else {
                        b.quad(c[0], c[3], c[2], c[1], col);
                    }
                }
            }
        }
    }
}

/// A house in a continuous frontage: the full width of its plot, two or three
/// storeys, its own colour and roof, so a high street reads as a row of separate houses.
fn townhouse(houses: &mut Builder, smoke: &mut Builder, g: &Ground, pl: &Plot, dens: f32) {
    let p = ground(g, pl.p.x, pl.p.y);
    let storeys = 2.0 + ((dens > 0.7 && pl.v > 0.55) as u8 as f32);
    let w = pl.width * 0.99;
    let d = pl.depth * (0.95 + 0.15 * pl.u);
    let wall = if pl.v < 0.25 { hex(BRICK[(pl.u * 4.0) as usize % 4]) } else { hex(WALLS[(pl.v * 6.0) as usize % 6]) };
    let roof = hex(ROOFS[(pl.u * 13.0) as usize % 5]);
    let h = 0.024 * storeys;
    houses.block(p, w, d, h, pl.yaw, wall, None, 0.03);
    houses.gable(p, w, d, h, d * (0.4 + 0.2 * pl.u), pl.yaw, 0.004, roof, wall);
    windows(houses, p, w, d, storeys, pl.yaw, 1, true);
    let cp = p + bevy::math::Quat::from_rotation_y(pl.yaw) * Vec3::new(w * 0.42, 0.0, 0.0);
    if pl.u < 0.12 {
        smoking_chimney(houses, smoke, cp, h + d * 0.3, 0.004, pl.v * 7.0);
    } else {
        chimney(houses, cp, h + d * 0.3, 0.004);
    }
}
