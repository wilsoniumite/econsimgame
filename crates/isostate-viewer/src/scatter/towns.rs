//! Towns, laid out the way market towns grew. There is a market place with the
//! church in its yard just off it, in a quarter between the roads. High streets run along the roads in, with an unbroken
//! frontage near the middle. Back lanes run behind the plots, with cross streets
//! between them; their houses go up a row at a time. A station stands at the end
//! of its own road.
//!
//! Every street, plot and row is fixed by where the roads run, and appears once
//! its ground is built up enough, so a town thickens in place from year to year.
//! What a building looks like is settled when it goes up and changes only in
//! steps of its own: a row built after 1815 is a brick terrace and stays one,
//! and each house adds a storey at its own density, not all of a street at once.
//! Houses outlast a small fall in their ground's density, so a town that shrinks
//! a little does not lose them.

use super::*;
use bevy::math::Vec2;
use isostate_gen::houses::{self, roof, storeys, Cottage, Roof, Terrace, Townhouse};
use isostate_gen::town;
use std::collections::HashMap;

/// Where a road enters town and what lies along it, in world units.
const BACK_LANE: f32 = 0.13;
const OUTER_LANE: f32 = 0.26;
const CROSS_EVERY: f32 = 0.17;

/// Rows on the back streets built from this year on are terraces.
const TERRACE_ERA: i32 = 1815;
/// How far a plot's ground may fall below what it needed before its house goes.
const KEEP: f32 = 0.12;

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
    /// the street and side it fronts and its place along it
    run: (i32, u8),
    /// the row it belongs to on a back street, built all at once (none on a high street)
    row: Option<i32>,
    /// it fronts a road through town: the old high street rather than a newer lane
    high: bool,
    index: i32,
    width: f32,
    depth: f32,
    need: f32,
    u: f32,
    v: f32,
}

/// The first year a plot's ground was town ground built up to `need`, if it has been by year k.
fn first_built(r: &Replay, k: usize, wc: usize, sub: f32, need: f32) -> Option<usize> {
    (0..=k).find(|&y| density(r, y, wc) >= need && matches!(world::pick_class(r, y, wc, sub), world::URBAN | world::WORKS))
}

/// The roof of a house that went up in year `built` (counted from the start)
/// on the ground of cell `wc`, as it is in year `k`: what could be had
/// there that year, weathered since. A house standing when the story opens
/// was built some time before.
fn roof_of(r: &Replay, k: usize, wc: usize, built: usize, seed: Seed) -> Roof {
    let (year, now) = (if built == 0 { roof::built_before(r.y0, seed) } else { r.y0 + built as i32 }, r.y0 + k as i32);
    // whether the railway had come by the year the roof it has now was laid
    let laid = (Roof::laid(year, now, seed) - r.y0).clamp(0, k as i32) as usize;
    Roof::of(year, now, r.terrain[wc] >= 2, r.trans[laid * r.n + wc] & 4 != 0, seed)
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

/// A town's parish church: which church it is, the middle of its churchyard
/// and the way it faces.
pub(super) struct Parish {
    church: town::Church,
    at: Vec2,
    yaw: f32,
    /// what is wrong with where it stands, if no quarter was without fault (see `parish`)
    faults: u32,
}

impl Parish {
    /// Whether a point is in the churchyard or within `margin` of it.
    pub(super) fn holds(&self, p: Vec2, margin: f32) -> bool {
        p.distance_squared(self.at) < 0.04 && self.church.yard_holds(self.at, self.yaw, p, margin)
    }

    /// Whether a house plot or a back street here would crowd the church or block the way to its gate.
    fn crowded_by(&self, p: Vec2) -> bool {
        p.distance_squared(self.at) < 0.06 && self.church.crowded_by(self.at, self.yaw, p)
    }
}

/// How much further out each try is for a church that cannot stand on dry ground by its market place.
const FURTHER_OUT: f32 = 0.12;

/// East, south, west and north on the map, in the order they come round.
const WAYS: [(isize, isize); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

/// A town's church stands off its market place, tower and gate toward it, in
/// one of the four quarters between the ways to the neighbouring squares of
/// the map (where the roads run, now or later), and stays there: the quarter
/// is chosen by what does not change. It must be dry land clear of those
/// ways, and of the quarters that are, the one that keeps the churchyard
/// nearest the middle of its square of the map is taken, as the works stand
/// round the edge. Only a road that cuts across a quarter, which is rare, is
/// a reason to take another, from the year it is first laid, and so is a
/// railway laid through the churchyard (the line goes where the simulation
/// has it, and the church gives way the year it is begun). A town with
/// water in every quarter stands its church in one of the ways out, and one
/// whose middle is drowned ground stands it further out the same way, on the
/// nearest dry ground.
fn parish(r: &Replay, g: &Ground, cen: &[(f32, f32)], rails: &crate::rails::RailYear, k: usize, s: &Square) -> Parish {
    let t = s.town;
    let node = r.header.towns[t].node;
    // a big town's church was rebuilt bigger, and stays so; a small town's gets its spire in a year of its own
    let grown = (0..=k).any(|y| r.town_urban[y * r.n_towns + t] > 12000.0);
    let spire_year = 1825 + (world::hash2(t as i32, 3, 5) * 50.0) as i32;
    let church = town::Church { big: grown, spire: grown || r.y0 + k as i32 >= spire_year, seed: Seed::new(world::hash2(t as i32, 4, 5), world::hash2(t as i32, 5, 5)) };
    let (cx, cy) = ((node % r.w) as isize, (node / r.w) as isize);
    let cell = |(dx, dy): (isize, isize)| -> Option<usize> {
        let (x, y) = (cx + dx, cy + dy);
        (x >= 0 && y >= 0 && (x as usize) < r.w && (y as usize) < r.h).then(|| y as usize * r.w + x as usize)
    };
    // the ways out: toward the middle of each neighbouring square
    let ways = WAYS.map(|w| {
        let plain = Vec2::new(w.0 as f32, w.1 as f32);
        cell(w).map_or(plain, |c| (v2(cen[c]) - s.at).normalize_or(plain))
    });
    let roaded = |c: Option<usize>, y: usize| c.is_some_and(|c| r.trans[y * r.n + c] & 2 != 0);
    let middle = Vec2::new(cx as f32 + 0.5, cy as f32 + 0.5);
    // where it stands is settled by the yard of the biggest church the town may come to have,
    // so that a church rebuilt bigger grows back from its gate and does not move to another quarter
    let widest = town::Church { big: true, ..church };
    let (w, d) = widest.yard();
    let mut best: Option<(f32, Vec2, f32, u32)> = None;
    // the four quarters, and then the four ways themselves, for a town with water in every
    // quarter; and where the church would stand in the water in all eight (a town whose middle
    // is drowned ground), the same again further and further out, to the nearest dry ground
    for (i, step) in (0..5).flat_map(|step| (0..8).map(move |i| (i, step))) {
        if i == 0 && best.is_some_and(|(_, _, _, faults)| faults & 8 == 0) {
            break;
        }
        let out = step as f32 * FURTHER_OUT;
        let on_a_way = i >= 4;
        let (a, b) = (ways[i % 4], ways[(i + 1) % 4]);
        let bearing = if on_a_way { a } else { (a + b).normalize_or(Vec2::new(-a.y, a.x)) };
        let (at, yaw) = widest.beside_square(s.at + bearing * out, bearing);
        let across = Vec2::new(-bearing.y, bearing.x);
        let corners = [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].map(|(sx, sz)| at + across * (sx * w * 0.5) + bearing * (sz * d * 0.5));
        let mut faults = 0;
        // dry land under the whole yard, and under the church above all
        let wet = |p: &Vec2| world::height_at(g, p.x, p.y) < 0.05 || world::water_dist(g, p.x, p.y) < 0.03;
        let wet_corners = corners.iter().filter(|p| wet(p)).count();
        if wet_corners > 0 {
            faults += 2;
        }
        if wet(&at) {
            faults += 8;
        }
        if step > 0 {
            faults += 32;
        }
        if railway_through(rails, &widest, at, yaw, 0.01) {
            faults += 64;
        }
        if on_a_way {
            faults += 16;
        } else {
            // clear of the ways either side of it, where a main road and its frontage may come
            if corners.iter().any(|p| [a, b].iter().any(|way| (*p - s.at).dot(*way) > 0.0 && (*p - s.at).perp_dot(*way).abs() < 0.024)) {
                faults += 1;
            }
            // a road across the quarter, to the square of the map cornerwise, in any year so far
            let (da, db) = (WAYS[i], WAYS[(i + 1) % 4]);
            let corner = cell((da.0 + db.0, da.1 + db.1));
            if (0..=k).any(|y| roaded(Some(node), y) && roaded(corner, y) && !roaded(cell(da), y) && !roaded(cell(db), y)) {
                faults += 4;
            }
        }
        let reach = corners.iter().map(|p| (*p - middle).abs().max_element()).fold(0.0f32, f32::max);
        // the church in the water is the worst of the faults, then standing in a way out, the
        // railway through its yard, a road across its quarter, and a way close by; a corner of
        // the yard by the water is the least
        let weigh = |bit: u32, by: f32| if faults & bit != 0 { by } else { 0.0 };
        let score = weigh(8, 100.0) + weigh(16, 60.0) + weigh(64, 50.0) + weigh(4, 40.0) + weigh(1, 10.0) + wet_corners as f32 * 2.0 + reach;
        if best.map_or(true, |(sc, _, _, _)| score < sc) {
            best = Some((score, bearing, out, faults));
        }
    }
    let (_, bearing, out, faults) = best.unwrap_or((0.0, Vec2::X, 0.0, 0));
    let (at, yaw) = church.beside_square(s.at + bearing * out, bearing);
    Parish { church, at, yaw, faults }
}

/// Whether the railway, laid or being laid, runs through a churchyard whose
/// middle is at `at`, or within `margin` of it.
fn railway_through(rails: &crate::rails::RailYear, church: &town::Church, at: Vec2, yaw: f32, margin: f32) -> bool {
    let through = |a: Vec2, b: Vec2| {
        // a length of line far from the yard is passed over; one near it is walked in short steps
        let len = a.distance(b);
        if a.distance(at) > len + 0.25 {
            return false;
        }
        let steps = (len / 0.02).ceil().max(1.0) as usize;
        (0..=steps).any(|i| church.yard_holds(at, yaw, a.lerp(b, i as f32 / steps as f32), margin))
    };
    rails.lines.iter().any(|l| l.pts.windows(2).any(|w| through(Vec2::new(w[0].x, w[0].z), Vec2::new(w[1].x, w[1].z)))) || rails.building.iter().any(|&(a, b)| through(a, b))
}

/// The station is down its own road to the south of the market place.
fn station_at(s: &Square) -> Vec2 {
    let q = bevy::math::Quat::from_rotation_y(s.yaw);
    let o = q * Vec3::new(0.0, 0.0, 0.24);
    s.at + Vec2::new(o.x, o.z)
}

/// The market places, churches, stations and lighthouses of the towns. The
/// churchyards are handed back, for everything else to keep out of. `plots`
/// holds the yards of the year's works as it comes in, and is given the
/// market places and the stations.
pub(super) fn landmarks(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, lk: &Look, houses: &mut Builder, plots: &mut HashMap<usize, Vec<(f32, f32, f32, f32)>>) -> Vec<Parish> {
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    let mut parishes = Vec::new();
    let rails = crate::rails::year(r, g, cen, k);
    for s in squares(r, cen, k) {
        let node = r.header.towns[s.town].node;
        // the market place, paved, with a cross or a hall in the middle
        let paving = *lk.ground.market_paving;
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
            town::market_hall(houses, &Site::level(m, s.yaw), lk);
        } else {
            town::market_cross(houses, &Site::level(m, s.yaw), lk);
        }
        let p = parish(r, g, cen, &rails, k, &s);
        town::church(houses, &Site::on(ground(g, p.at.x, p.at.y), p.yaw, &height), &p.church, lk);
        if std::env::var_os("ISOSTATE_TIMING").is_some() {
            // a works whose yard reaches into the churchyard (so far only the yards of the works are in `plots`)
            let crowded = plots.values().flatten().filter(|&&(x, z, hx, hz)| p.holds(Vec2::new(p.at.x.clamp(x - hx, x + hx), p.at.y.clamp(z - hz, z + hz)), 0.0)).count();
            if p.faults > 0 || crowded > 0 {
                eprintln!("year {}: the church of town {} stands with faults {} (1 by a way out, 2 a corner of the yard on wet ground, 4 a road across, 8 the church on wet ground, 16 in a way out, 32 further out than the market place's edge, 64 the railway through the yard it may come to have) and {} works by its yard; it is at {:.2}, {:.2}, where the ground is {:.3} high and the water {:.3} off", r.y0 + k as i32, s.town, p.faults, crowded, p.at.x, p.at.y, world::height_at(g, p.at.x, p.at.y), world::water_dist(g, p.at.x, p.at.y));
            }
        }
        parishes.push(p);
        plots.entry(node).or_default().push((s.at.x, s.at.y, 0.085, 0.085));
        if r.trans[k * r.n + node] & 4 != 0 {
            let st = station_at(&s);
            town::station(houses, &Site::level(ground(g, st.x, st.y), s.yaw), lk);
            plots.entry(node).or_default().push((st.x, st.y, 0.085, 0.085));
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
            town::lighthouse(houses, &Site::level(ground(g, lx, lz), 0.0), lk);
        }
    }
    parishes
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
pub(super) fn houses(r: &Replay, g: &Ground, slots: &Slots, cen: &[(f32, f32)], k: usize, lk: &Look, parishes: &[Parish], near_plot: &dyn Fn(f32, f32) -> bool, houses: &mut Builder, smoke: &mut Builder) {
    let sq = squares(r, cen, k);
    // a house keeps its whole plot out of a churchyard
    let in_square = |p: Vec2| sq.iter().any(|s| p.distance(s.at) < s.radius + 0.012 || p.distance(station_at(s)) < 0.07) || parishes.iter().any(|c| c.crowded_by(p));
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
    let colour = *lk.ground.street_earth;
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

    // plots along both sides of every laid street, at fixed places along it, then round each market place
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
        let count = (len / spacing).floor() as i32;
        // a plot's place is fixed along its street: the i-th from the street's start
        let t_of = |i: i32| (i as f32 + 0.5) * spacing / len;
        let laid_at = |t: f32| runs.iter().any(|&(t0, t1)| t >= t0 && t <= t1);
        for (j, side) in [(0u8, -1.0f32), (1, 1.0)] {
            let id = s.id * 4 + j as i32;
            // the back streets are built a row at a time: rows of three to six plots,
            // fixed per street, each with one need, so a row goes up all together
            let mut rows: Vec<(i32, i32, i32)> = Vec::new();
            if !s.road {
                let mut i = 0;
                while i < count {
                    let l = 3 + (world::hash2(id, rows.len() as i32, 28) * 4.0) as i32;
                    rows.push((rows.len() as i32, i, (i + l).min(count)));
                    i += l;
                }
            }
            for i in 0..count {
                let row = rows.iter().find(|&&(_, a, b)| i >= a && i < b).copied();
                // a row comes in only where its whole length is laid
                let whole = match row {
                    Some((_, a, b)) => (a..b).all(|q| laid_at(t_of(q))),
                    None => laid_at(t_of(i)),
                };
                if !whole {
                    continue;
                }
                let (pu, pv) = (world::hash2(id, i, 24), world::hash2(id, i, 25));
                // a plot (or its row) comes in once its ground is built up beyond the street's own need
                let need = match row {
                    Some((ri, _, _)) => s.need + (1.0 - s.need) * world::hash2(id, ri, 29) * 0.95,
                    None => s.need + (1.0 - s.need) * pu * 0.75,
                };
                cands.push(Plot {
                    p: s.a + along * t_of(i) + n * side * set_back,
                    yaw: yaw_along + if side < 0.0 { std::f32::consts::PI } else { 0.0 },
                    run: (id, j),
                    row: row.map(|r| r.0),
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
    for s in &sq {
        // houses round the market place, facing in
        let count = ((std::f32::consts::TAU * (s.radius + 0.03)) / 0.05).floor() as i32;
        for i in 0..count {
            let a = i as f32 / count as f32 * std::f32::consts::TAU;
            let dirv = Vec2::new(a.cos(), a.sin());
            let p = s.at + dirv * (s.radius + 0.03);
            let id = -1000 - s.town as i32;
            let pu = world::hash2(id, i, 26);
            cands.push(Plot { p, yaw: (dirv.y).atan2(-dirv.x) + std::f32::consts::FRAC_PI_2, run: (id, 0), row: None, high: true, index: i, width: 0.05, depth: 0.036, need: 0.55 * pu, u: pu, v: world::hash2(id, i, 27) });
        }
    }

    // the plots that have been built and still stand, earliest first: they keep their
    // ground as the town grows. Each knows the year it was built, which settles its style.
    cands.sort_by(|a, b| a.need.total_cmp(&b.need));
    let mut taken: HashMap<(i32, i32), Vec<Vec2>> = HashMap::new();
    let mut standing: Vec<(Plot, f32, i32, Roof)> = Vec::new();
    let mut row_year: HashMap<(i32, i32), Option<usize>> = HashMap::new();
    for pl in cands {
        let (cls, wc) = class_here(r, g, k, pl.p.x, pl.p.y);
        if cls != world::URBAN || world::height_at(g, pl.p.x, pl.p.y) < 0.05 {
            continue;
        }
        let dens = density(r, k, wc);
        if dens < pl.need - KEEP || near_plot(pl.p.x, pl.p.y) || in_square(pl.p) || on_street(pl.p, 0.012) {
            continue;
        }
        let sub = world::sub_uniform(g, pl.p.x, pl.p.y) * 255.0;
        // a row's year is its first plot's, so the whole row shares one style
        let built = match pl.row {
            Some(ri) => *row_year.entry((pl.run.0, ri)).or_insert_with(|| first_built(r, k, wc, sub, pl.need)),
            None => first_built(r, k, wc, sub, pl.need),
        };
        let Some(built) = built else { continue };
        let c = cell(pl.p);
        if (-1..=1).any(|dy| (-1..=1).any(|dx| taken.get(&(c.0 + dx, c.1 + dy)).is_some_and(|v| v.iter().any(|q| q.distance(pl.p) < 0.042)))) {
            continue;
        }
        taken.entry(c).or_default().push(pl.p);
        let roof = roof_of(r, k, wc, built, Seed::new(pl.u, pl.v));
        standing.push((pl, dens, r.y0 + built as i32, roof));
    }

    // rows built in the industrial years are brick terraces; everything else stands as separate houses
    standing.sort_by_key(|(p, _, _, _)| (p.run, p.index));
    let mut i = 0;
    while i < standing.len() {
        let (first, _, year) = (&standing[i].0, standing[i].1, standing[i].2);
        let mut j = i + 1;
        if let (Some(ri), true) = (first.row, year >= TERRACE_ERA) {
            while j < standing.len() && standing[j].0.run == first.run && standing[j].0.row == Some(ri) && standing[j].0.index == standing[j - 1].0.index + 1 {
                j += 1;
            }
        }
        if j - i >= 2 {
            terrace(houses, smoke, lk, g, &standing[i..j]);
        } else {
            for (pl, d, _, roof) in &standing[i..j] {
                // on the high streets and the market place a house is rebuilt as a
                // townhouse of the full plot once its ground is dense enough, each at its own point
                if pl.high && *d >= pl.need + 0.12 + 0.25 * pl.v {
                    townhouse(houses, smoke, lk, g, pl, *d, *roof);
                } else {
                    cottage(houses, smoke, lk, g, pl, *d, *roof);
                }
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
        let pl = Plot { p, yaw: (s.yaw / std::f32::consts::FRAC_PI_2).round() * std::f32::consts::FRAC_PI_2, run: (0, 0), row: None, high: false, index: 0, width: 0.05, depth: 0.036, need: 0.0, u: s.u, v: s.v };
        let built = first_built(r, k, c, s.sub, 0.0).unwrap_or(k);
        cottage(houses, smoke, lk, g, &pl, density(r, k, c), roof_of(r, k, c, built, Seed::new(s.u, s.v)));
    }
}

/// A house standing on its own plot. Each adds a storey at its own density,
/// so a street grows a house at a time.
fn cottage(houses: &mut Builder, smoke: &mut Builder, lk: &Look, g: &Ground, pl: &Plot, dens: f32, roof: Roof) {
    let seed = Seed::new(pl.u, pl.v);
    let c = Cottage { plot_width: pl.width, plot_depth: pl.depth, storeys: storeys::cottage(dens, seed), roof, seed };
    houses::cottage(houses, smoke, &Site::level(ground(g, pl.p.x, pl.p.y), pl.yaw), &c, lk);
}

/// A terraced row over several plots, raised a storey when its own ground is dense enough.
fn terrace(houses: &mut Builder, smoke: &mut Builder, lk: &Look, g: &Ground, row: &[(Plot, f32, i32, Roof)]) {
    let (a, b) = (row[0].0.p, row[row.len() - 1].0.p);
    let mid = (a + b) * 0.5;
    let first = &row[0].0;
    let dens = row.iter().map(|(_, d, _, _)| *d).sum::<f32>() / row.len() as f32;
    // ground under the row: the lower end, so it never floats
    let h = row.iter().map(|(pl, _, _, _)| world::height_at(g, pl.p.x, pl.p.y)).fold(f32::MAX, f32::min);
    let seed = Seed::new(first.u, first.v);
    let t = Terrace { houses: row.len(), length: a.distance(b) + first.width * 0.96, depth: first.depth, storeys: storeys::terrace(dens, seed), roof: row[0].3, seed };
    houses::terrace(houses, smoke, &Site::level(Vec3::new(mid.x, h, mid.y), first.yaw), &t, lk);
}

/// A house in a continuous frontage: the full width of its plot, two or three storeys.
fn townhouse(houses: &mut Builder, smoke: &mut Builder, lk: &Look, g: &Ground, pl: &Plot, dens: f32, roof: Roof) {
    let seed = Seed::new(pl.u, pl.v);
    let t = Townhouse { plot_width: pl.width, plot_depth: pl.depth, storeys: storeys::townhouse(dens, seed), roof, seed };
    houses::townhouse(houses, smoke, &Site::level(ground(g, pl.p.x, pl.p.y), pl.yaw), &t, lk);
}
