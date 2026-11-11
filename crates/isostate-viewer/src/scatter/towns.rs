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
use isostate_gen::houses::{self, roof, storeys, Cottage, Era, Roof, Terrace, Townhouse, Villa};
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
    pub(super) at: Vec2,
    radius: f32,
    pub(super) yaw: f32,
    pub(super) town: usize,
    big: bool,
}

/// A house plot fronting a street.
struct Plot {
    p: Vec2,
    pub(super) yaw: f32,
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

/// A plot with a house on it this year.
struct Standing {
    pl: Plot,
    /// how built up its ground is now
    dens: f32,
    /// the year (counted from the start) its ground was first built up enough for it
    built: usize,
    roof: Roof,
    /// the cell and the patch of it the plot's ground is read from
    wc: usize,
    sub: f32,
    /// a house stands on the next plot along its street, on its left and on its right as it faces the street
    neighbours: (bool, bool),
}

/// The year a house that went up in year `built` (counted from the start) was
/// built: one standing when the story opens was built some time before.
fn year_built(r: &Replay, built: usize, seed: Seed) -> i32 {
    if built == 0 { roof::built_before(r.y0, seed) } else { r.y0 + built as i32 }
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
    let (year, now) = (year_built(r, built, seed), r.y0 + k as i32);
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
    pub(super) at: Vec2,
    pub(super) yaw: f32,
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
/// has it, and the church gives way the year it is begun, and does not come
/// back if the line as built misses the yard after all). A town with
/// water in every quarter stands its church in one of the ways out, and one
/// whose middle is drowned ground stands it further out the same way, on the
/// nearest dry ground.
///
/// `through` says which of the spots a church might take the railway has run
/// through in any year so far (`rail_hits`, gathered by `Railways`).
fn parish(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, through: u64, s: &Square) -> Parish {
    let t = s.town;
    // a big town's church was rebuilt bigger, and stays so; a small town's gets its spire in a year of its own
    let grown = (0..=k).any(|y| r.town_urban[y * r.n_towns + t] > 12000.0);
    let spire_year = 1825 + (world::hash2(t as i32, 3, 5) * 50.0) as i32;
    let church = town::Church { big: grown, spire: grown || r.y0 + k as i32 >= spire_year, seed: Seed::new(world::hash2(t as i32, 4, 5), world::hash2(t as i32, 5, 5)) };
    let (bearing, out, faults) = stand(r, g, cen, k, through, s);
    let (at, yaw) = church.beside_square(s.at + bearing * out, bearing);
    Parish { church, at, yaw, faults }
}

/// The spots a town's church might take: the four quarters and the four ways
/// out, and the same again further and further out.
const SPOTS: usize = 8;
const STEPS: usize = 5;

/// The church whose yard settles where a town's church stands: the biggest
/// the town may come to have, so that a church rebuilt bigger grows back from
/// its gate and does not move to another quarter.
fn widest() -> town::Church {
    town::Church { big: true, spire: true, seed: Seed::new(0.0, 0.0) }
}

/// The ways out of a town: toward the middle of each neighbouring square of the map.
fn ways_out(r: &Replay, cen: &[(f32, f32)], t: usize) -> [Vec2; 4] {
    let node = r.header.towns[t].node;
    let (cx, cy) = ((node % r.w) as isize, (node / r.w) as isize);
    WAYS.map(|w| {
        let plain = Vec2::new(w.0 as f32, w.1 as f32);
        let (x, y) = (cx + w.0, cy + w.1);
        let there = (x >= 0 && y >= 0 && (x as usize) < r.w && (y as usize) < r.h).then(|| y as usize * r.w + x as usize);
        there.map_or(plain, |c| (v2(cen[c]) - v2(cen[node])).normalize_or(plain))
    })
}

/// Spot `i` at `step` steps out: the way it lies from the market place, how
/// much further out than the market place's edge, the middle of the yard
/// the widest church would have there, and its yaw.
fn spot(ways: &[Vec2; 4], square: Vec2, i: usize, step: usize) -> (Vec2, f32, Vec2, f32) {
    let out = step as f32 * FURTHER_OUT;
    let (a, b) = (ways[i % 4], ways[(i + 1) % 4]);
    let bearing = if i >= 4 { a } else { (a + b).normalize_or(Vec2::new(-a.y, a.x)) };
    let (at, yaw) = widest().beside_square(square + bearing * out, bearing);
    (bearing, out, at, yaw)
}

/// What has been worked out about where the churches stand, kept because the
/// works, which must know every churchyard of the record before they take
/// their ground, ask for every year at once, and most years ask again for
/// what an earlier one found. `hits`: the spots of a town the railway of one
/// network runs through, by the town and the network (`rails::key`).
/// `stands`: where a town's church stands, by the town, the spots the railway
/// has run through so far, and the quarters a road has cut across so far.
#[derive(Default)]
struct Known {
    hits: HashMap<(usize, [u32; 2], u64), u64>,
    stands: HashMap<(usize, [u32; 2], u64, u8), (Vec2, f32, u32)>,
}

static KNOWN: std::sync::Mutex<Option<Known>> = std::sync::Mutex::new(None);

/// Which of town `t`'s spots the railway of year `y` (whose network is
/// `rail_key`), laid or being laid, runs through: a bit for each.
fn rail_hits(r: &Replay, g: &Ground, cen: &[(f32, f32)], y: usize, rail_key: u64, t: usize) -> u64 {
    let square = v2(cen[r.header.towns[t].node]);
    let key = (t, [square.x.to_bits(), square.y.to_bits()], rail_key);
    if let Some(&known) = KNOWN.lock().unwrap().as_ref().and_then(|m| m.hits.get(&key)) {
        return known;
    }
    // the furthest spot's yard ends well within a cell of the market place
    let near = railway_near(&crate::rails::year(r, g, cen, y), square, 1.3);
    let mut hits = 0u64;
    if !near.is_empty() {
        let ways = ways_out(r, cen, t);
        for step in 0..STEPS {
            for i in 0..SPOTS {
                let (_, _, at, yaw) = spot(&ways, square, i, step);
                if railway_through(&near, &widest(), at, yaw, 0.01) {
                    hits |= 1 << (step * SPOTS + i);
                }
            }
        }
    }
    KNOWN.lock().unwrap().get_or_insert_with(Known::default).hits.insert(key, hits);
    hits
}

/// The railways of the years up to one, for the churches: which spots of
/// each town a line has run through in any of those years.
pub(super) struct Railways {
    through: Vec<u64>,
    last: Option<u64>,
}

impl Railways {
    pub(super) fn new(r: &Replay) -> Railways {
        Railways { through: vec![0; r.header.towns.len()], last: None }
    }

    /// Take in year `y`. The years are to be given in order, from the first.
    pub(super) fn add(&mut self, r: &Replay, g: &Ground, cen: &[(f32, f32)], y: usize) {
        let key = crate::rails::key(r, y);
        if self.last == Some(key) {
            return;
        }
        self.last = Some(key);
        for (t, through) in self.through.iter_mut().enumerate() {
            *through |= rail_hits(r, g, cen, y, key, t);
        }
    }

    /// As they stand after every year up to `k`.
    pub(super) fn up_to(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize) -> Railways {
        let mut all = Railways::new(r);
        for y in 0..=k {
            all.add(r, g, cen, y);
        }
        all
    }
}

/// Which way from its market place a town's church stands in year `k`, how
/// much further out than the market place's edge, and what is wrong with the
/// spot (see `parish`).
fn stand(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, through: u64, s: &Square) -> (Vec2, f32, u32) {
    let node = r.header.towns[s.town].node;
    let (cx, cy) = ((node % r.w) as isize, (node / r.w) as isize);
    let cell = |(dx, dy): (isize, isize)| -> Option<usize> {
        let (x, y) = (cx + dx, cy + dy);
        (x >= 0 && y >= 0 && (x as usize) < r.w && (y as usize) < r.h).then(|| y as usize * r.w + x as usize)
    };
    let roaded = |c: Option<usize>, y: usize| c.is_some_and(|c| r.trans[y * r.n + c] & 2 != 0);
    // a road across each quarter, to the square of the map cornerwise, in any year so far
    let cut: [bool; 4] = std::array::from_fn(|i| {
        let (da, db) = (WAYS[i], WAYS[(i + 1) % 4]);
        let corner = cell((da.0 + db.0, da.1 + db.1));
        (0..=k).any(|y| roaded(Some(node), y) && roaded(corner, y) && !roaded(cell(da), y) && !roaded(cell(db), y))
    });
    let key = (s.town, [s.at.x.to_bits(), s.at.y.to_bits()], through, cut.iter().enumerate().fold(0u8, |bits, (i, &c)| bits | (c as u8) << i));
    if let Some(&known) = KNOWN.lock().unwrap().as_ref().and_then(|m| m.stands.get(&key)) {
        return known;
    }
    let ways = ways_out(r, cen, s.town);
    let middle = Vec2::new(cx as f32 + 0.5, cy as f32 + 0.5);
    let (w, d) = widest().yard();
    let mut best: Option<(f32, Vec2, f32, u32)> = None;
    // the four quarters, and then the four ways themselves, for a town with water in every
    // quarter; and where the church would stand in the water in all eight (a town whose middle
    // is drowned ground), the same again further and further out, to the nearest dry ground
    for (i, step) in (0..STEPS).flat_map(|step| (0..SPOTS).map(move |i| (i, step))) {
        if i == 0 && best.is_some_and(|(_, _, _, faults)| faults & 8 == 0) {
            break;
        }
        let on_a_way = i >= 4;
        let (a, b) = (ways[i % 4], ways[(i + 1) % 4]);
        let (bearing, out, at, _) = spot(&ways, s.at, i, step);
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
        if through & 1 << (step * SPOTS + i) != 0 {
            faults += 64;
        }
        if on_a_way {
            faults += 16;
        } else {
            // clear of the ways either side of it, where a main road and its frontage may come
            if corners.iter().any(|p| [a, b].iter().any(|way| (*p - s.at).dot(*way) > 0.0 && (*p - s.at).perp_dot(*way).abs() < 0.024)) {
                faults += 1;
            }
            if cut[i] {
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
    KNOWN.lock().unwrap().get_or_insert_with(Known::default).stands.insert(key, (bearing, out, faults));
    (bearing, out, faults)
}

/// The way from a churchyard's gate to its market place, which a works keeps
/// out of as a house does (`Church::crowded_by`).
const GATE_WAY: f32 = 0.07;

/// The ground a town's church keeps, and the years it keeps it: the works take
/// theirs clear of it.
pub(super) struct Churchyard {
    pub(super) town: usize,
    church: town::Church,
    pub(super) at: Vec2,
    pub(super) yaw: f32,
    /// the first year (counted from the start) it stands here, and one past the
    /// last, or no end if it stands here in the last year known
    pub(super) from: usize,
    pub(super) to: usize,
}

impl Churchyard {
    /// Whether a box on the map (its middle, and half its width and depth) meets
    /// the churchyard or the way to its gate.
    pub(super) fn meets_box(&self, x: f32, z: f32, hx: f32, hz: f32) -> bool {
        yard_meets_box(&self.church, self.at, self.yaw, GATE_WAY, x, z, hx, hz)
    }

    /// A number that changes when the churchyard is another, or begins in another year.
    pub(super) fn mark(&self) -> u64 {
        [self.town as u64, self.at.x.to_bits() as u64, self.at.y.to_bits() as u64, self.yaw.to_bits() as u64, self.from as u64].iter().fold(1469598103934665603u64, |h, &b| (h ^ b).wrapping_mul(1099511628211))
    }
}

/// Whether a box on the map meets a churchyard whose middle is at `at`, with
/// `way` of clear ground before its gate. The yard stands askew to the map, so
/// the two are tried along the sides of each (they are apart if any side
/// shows a gap between them).
#[allow(clippy::too_many_arguments)]
fn yard_meets_box(church: &town::Church, at: Vec2, yaw: f32, way: f32, x: f32, z: f32, hx: f32, hz: f32) -> bool {
    let (w, depth) = church.yard();
    let (sin, cos) = yaw.sin_cos();
    // the yard's own x and z on the map (`Church::yard_holds`); its front, where the gate is, is +z
    let (ex, ez) = (Vec2::new(cos, -sin), Vec2::new(sin, cos));
    let (bw, bd) = (w * 0.5, (depth + way) * 0.5);
    let d = at + ez * (way * 0.5) - Vec2::new(x, z);
    [Vec2::X, Vec2::Y, ex, ez].iter().all(|n| d.dot(*n).abs() <= hx * n.x.abs() + hz * n.y.abs() + bw * n.dot(ex).abs() + bd * n.dot(ez).abs())
}

/// Every churchyard of the record so far, with the years each is kept: a
/// town's church stands where it stood unless a road or the railway moved it,
/// and is rebuilt bigger when its town has grown.
pub(super) fn churchyards(r: &Replay, g: &Ground, cen: &[(f32, f32)]) -> Vec<Churchyard> {
    let last = r.filled.clamp(1, r.ny);
    let mut out: Vec<Churchyard> = Vec::new();
    let mut railways = Railways::new(r);
    for k in 0..last {
        railways.add(r, g, cen, k);
        for s in squares(r, cen, k) {
            let p = parish(r, g, cen, k, railways.through[s.town], &s);
            match out.iter_mut().rev().find(|y| y.town == s.town) {
                Some(y) if y.to == k && y.at == p.at && y.yaw == p.yaw && y.church.big == p.church.big => y.to = k + 1,
                _ => out.push(Churchyard { town: s.town, church: p.church, at: p.at, yaw: p.yaw, from: k, to: k + 1 }),
            }
        }
    }
    for y in &mut out {
        if y.to == last {
            y.to = usize::MAX;
        }
    }
    out
}

/// The lengths of railway, laid or being laid, that come within `reach` of a point.
fn railway_near(rails: &crate::rails::RailYear, p: Vec2, reach: f32) -> Vec<(Vec2, Vec2)> {
    let laid = rails.lines.iter().flat_map(|l| l.pts.windows(2).map(|w| (Vec2::new(w[0].x, w[0].z), Vec2::new(w[1].x, w[1].z))));
    laid.chain(rails.building.iter().copied()).filter(|&(a, b)| a.distance(p) <= a.distance(b) + reach).collect()
}

/// Whether one of these lengths of railway runs through a churchyard whose
/// middle is at `at`, or within `margin` of it.
fn railway_through(lengths: &[(Vec2, Vec2)], church: &town::Church, at: Vec2, yaw: f32, margin: f32) -> bool {
    lengths.iter().any(|&(a, b)| {
        // a length of line far from the yard is passed over; one near it is walked in short steps
        let len = a.distance(b);
        if a.distance(at) > len + 0.25 {
            return false;
        }
        let steps = (len / 0.02).ceil().max(1.0) as usize;
        (0..=steps).any(|i| church.yard_holds(at, yaw, a.lerp(b, i as f32 / steps as f32), margin))
    })
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
// (task B, one scale of detail: the landmarks take their level from the camera, as the houses do: `lod`)
pub(super) fn landmarks(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, lk: &Look, lod: &Lod, houses: &mut Builder, plots: &mut HashMap<usize, Vec<(f32, f32, f32, f32)>>) -> Vec<Parish> {
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    let mut parishes = Vec::new();
    let railways = Railways::up_to(r, g, cen, k);
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
            town::market_hall(houses, &Site::level(m, s.yaw), lod.at(m.x, m.z), lk);
        } else {
            town::market_cross(houses, &Site::level(m, s.yaw), lod.at(m.x, m.z), lk);
        }
        let p = parish(r, g, cen, k, railways.through[s.town], &s);
        town::church(houses, &Site::on(ground(g, p.at.x, p.at.y), p.yaw, &height), &p.church, lod.at(p.at.x, p.at.y), lk);
        if std::env::var_os("ISOSTATE_TIMING").is_some() {
            // a works whose yard (and the room kept round it) reaches into the churchyard: the works
            // take their ground clear of every churchyard (`churchyards`), so there should be none
            // but those that found no ground of their own. So far `plots` holds the works' yards
            // and the market places and stations of the towns before this one
            let crowded = plots.values().flatten().filter(|&&(x, z, hx, hz)| yard_meets_box(&p.church, p.at, p.yaw, 0.0, x, z, hx - 0.02, hz - 0.02)).count();
            if p.faults > 0 || crowded > 0 {
                eprintln!("year {}: the church of town {} stands with faults {} (1 by a way out, 2 a corner of the yard on wet ground, 4 a road across, 8 the church on wet ground, 16 in a way out, 32 further out than the market place's edge, 64 the railway through the yard it may come to have) and {} works in its yard; it is at {:.2}, {:.2}, where the ground is {:.3} high and the water {:.3} off", r.y0 + k as i32, s.town, p.faults, crowded, p.at.x, p.at.y, world::height_at(g, p.at.x, p.at.y), world::water_dist(g, p.at.x, p.at.y));
            }
        }
        parishes.push(p);
        plots.entry(node).or_default().push((s.at.x, s.at.y, 0.085, 0.085));
        if r.trans[k * r.n + node] & 4 != 0 {
            let st = station_at(&s);
            town::station(houses, &Site::level(ground(g, st.x, st.y), s.yaw), lod.at(st.x, st.y), lk);
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
            town::lighthouse(houses, &Site::level(ground(g, lx, lz), 0.0), lod.at(lx, lz), lk);
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

/// What the houses are built into, and how much of each is drawn where.
pub(super) struct Into<'a> {
    pub houses: &'a mut Builder,
    pub smoke: &'a mut Builder,
    pub trees: &'a mut [Greenery; LEAF_GROUPS],
    pub lod: &'a Lod,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn houses(r: &Replay, g: &Ground, slots: &Slots, cen: &[(f32, f32)], k: usize, lk: &Look, parishes: &[Parish], near_plot: &dyn Fn(f32, f32) -> bool, into: &mut Into) {
    let houses = &mut *into.houses;
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
                    // its front to the street
                    yaw: yaw_along + if side > 0.0 { std::f32::consts::PI } else { 0.0 },
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
    let mut standing: Vec<Standing> = Vec::new();
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
        standing.push(Standing { pl, dens, built, roof, wc, sub, neighbours: (false, false) });
    }

    // rows built in the industrial years are brick terraces; everything else stands as separate houses
    standing.sort_by_key(|s| (s.pl.run, s.pl.index));
    // which houses have a neighbour on the next plot along: a plot on the far side of its street
    // (and one round a market place) has the next along on its right, one on the near side on its left
    for i in 0..standing.len() {
        let next_to = |j: usize| standing[j].pl.run == standing[i].pl.run && (standing[j].pl.index - standing[i].pl.index).abs() == 1;
        let (before, after) = (i > 0 && next_to(i - 1), i + 1 < standing.len() && next_to(i + 1));
        standing[i].neighbours = if standing[i].pl.run.1 == 1 { (after, before) } else { (before, after) };
    }
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    let mut i = 0;
    while i < standing.len() {
        let first = &standing[i];
        let mut j = i + 1;
        if let (Some(ri), true) = (first.pl.row, r.y0 + first.built as i32 >= TERRACE_ERA) {
            while j < standing.len() && standing[j].pl.run == first.pl.run && standing[j].pl.row == Some(ri) && standing[j].pl.index == standing[j - 1].pl.index + 1 {
                j += 1;
            }
        }
        if j - i >= 2 {
            terrace(into, r, lk, g, &height, &standing[i..j]);
        } else {
            for s in &standing[i..j] {
                // on the high streets and the market place a house is rebuilt as a
                // townhouse of the full plot once its ground is dense enough, each at its own point
                let rebuilt = s.pl.need + 0.12 + 0.25 * s.pl.v;
                if s.pl.high && s.dens >= rebuilt {
                    townhouse(into, r, lk, g, &height, k, s, rebuilt);
                } else {
                    cottage(into, r, lk, g, &height, &s.pl, s.dens, s.built, s.roof, s.wc, s.neighbours);
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
        cottage(into, r, lk, g, &height, &pl, density(r, k, c), built, roof_of(r, k, c, built, Seed::new(s.u, s.v)), c, (false, false));
    }
}

/// A house standing on its own plot, in the style of the year it was first
/// built: a cottage, which adds a storey at its own density, so a street
/// grows a house at a time; or, on some of the plots first built once the
/// railway years' money was about, a villa in its garden.
#[allow(clippy::too_many_arguments)]
fn cottage(into: &mut Into, r: &Replay, lk: &Look, g: &Ground, height: &dyn Fn(f32, f32) -> f32, pl: &Plot, dens: f32, built: usize, roof: Roof, wc: usize, neighbours: (bool, bool)) {
    let seed = Seed::new(pl.u, pl.v);
    let year = year_built(r, built, seed);
    let (era, upland, detail) = (Era::of(year), r.terrain[wc] >= 2, into.lod.at(pl.p.x, pl.p.y));
    let site = Site::on(ground(g, pl.p.x, pl.p.y), pl.yaw, height);
    let green = &mut into.trees[leaf_group(pl.v)];
    if pl.row.is_none() && houses::villa_plot(year, seed) {
        let v = Villa { plot_width: pl.width, plot_depth: pl.depth, roof, era, upland, detail, seed };
        houses::villa(into.houses, into.smoke, green, &site, &v, lk);
    } else {
        let c = Cottage { plot_width: pl.width, plot_depth: pl.depth, storeys: storeys::cottage(dens, seed), roof, era, upland, neighbours, detail, seed };
        houses::cottage(into.houses, into.smoke, green, &site, &c, lk);
    }
}

/// A terraced row over several plots, raised a storey when its own ground is
/// dense enough. It is handed the ground, and steps with it.
fn terrace(into: &mut Into, r: &Replay, lk: &Look, g: &Ground, height: &dyn Fn(f32, f32) -> f32, row: &[Standing]) {
    let (a, b) = (row[0].pl.p, row[row.len() - 1].pl.p);
    let mid = (a + b) * 0.5;
    let first = &row[0];
    let dens = row.iter().map(|s| s.dens).sum::<f32>() / row.len() as f32;
    let seed = Seed::new(first.pl.u, first.pl.v);
    let t = Terrace {
        houses: row.len(),
        length: a.distance(b) + first.pl.width * 0.96,
        depth: first.pl.depth,
        storeys: storeys::terrace(dens, seed),
        roof: first.roof,
        era: Era::of(r.y0 + first.built as i32),
        upland: r.terrain[first.wc] >= 2,
        detail: into.lod.at(mid.x, mid.y),
        seed,
    };
    houses::terrace(into.houses, into.smoke, &Site::on(ground(g, mid.x, mid.y), first.pl.yaw, height), &t, lk);
}

/// A house in a continuous frontage: the full width of its plot, two or
/// three storeys, in the style of the year its ground was first dense
/// enough for it (`rebuilt` is that density), with a shop below where the
/// ground is busy enough.
#[allow(clippy::too_many_arguments)]
fn townhouse(into: &mut Into, r: &Replay, lk: &Look, g: &Ground, height: &dyn Fn(f32, f32) -> f32, k: usize, s: &Standing, rebuilt: f32) {
    let pl = &s.pl;
    let seed = Seed::new(pl.u, pl.v);
    let since = first_built(r, k, s.wc, s.sub, rebuilt).unwrap_or(s.built);
    let t = Townhouse {
        plot_width: pl.width,
        plot_depth: pl.depth,
        storeys: storeys::townhouse(s.dens, seed),
        roof: s.roof,
        era: Era::of(year_built(r, since, seed)),
        upland: r.terrain[s.wc] >= 2,
        shop: houses::shop_at(s.dens, seed),
        neighbours: s.neighbours,
        detail: into.lod.at(pl.p.x, pl.p.y),
        seed,
    };
    houses::townhouse(into.houses, into.smoke, &Site::on(ground(g, pl.p.x, pl.p.y), pl.yaw, height), &t, lk);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small town's church off a market place at nought, out to the south-east.
    fn yard() -> (town::Church, Vec2, f32, Vec2) {
        let church = town::Church { big: false, spire: false, seed: Seed::new(0.3, 0.7) };
        let bearing = Vec2::new(1.0, 1.0).normalize();
        let (at, yaw) = church.beside_square(Vec2::ZERO, bearing);
        (church, at, yaw, bearing)
    }

    #[test]
    fn a_point_meets_the_churchyard_where_the_church_says_its_yard_is() {
        let (church, at, yaw, _) = yard();
        let mut inside = 0;
        for j in 0..60 {
            for i in 0..60 {
                let p = at + Vec2::new(i as f32 - 29.5, j as f32 - 29.5) * 0.006;
                let holds = church.yard_holds(at, yaw, p, 0.0);
                assert_eq!(yard_meets_box(&church, at, yaw, 0.0, p.x, p.y, 0.0, 0.0), holds, "at {p}");
                inside += holds as usize;
            }
        }
        // the yard is neither missed nor everything
        assert!(inside > 200 && inside < 3000, "{inside} of 3600 points in the yard");
    }

    #[test]
    fn a_works_yard_keeps_out_of_the_churchyard_and_the_way_to_its_gate() {
        let (church, at, yaw, bearing) = yard();
        let (w, depth) = church.yard();
        let across = Vec2::new(-bearing.y, bearing.x);
        let meets = |p: Vec2, h: f32| yard_meets_box(&church, at, yaw, GATE_WAY, p.x, p.y, h, h);
        // on the church, over a corner of the yard, and across the way from the gate to the market place
        assert!(meets(at, 0.03));
        assert!(meets(at + across * (w * 0.5) + bearing * (depth * 0.5) + Vec2::new(0.02, 0.0), 0.03));
        assert!(meets(at - bearing * (depth * 0.5 + GATE_WAY * 0.5), 0.01));
        // the yard stands askew to the map: a box in the corner of the square drawn round it is clear of it
        let square = (w + depth) * 0.5 * std::f32::consts::FRAC_1_SQRT_2;
        assert!(!meets(at + Vec2::new(square - 0.006, -(square - 0.006)), 0.005));
        assert!(!meets(at + across * (w * 0.5 + 0.06), 0.02));
        // behind the chancel and beyond the market place's edge it is clear too
        assert!(!meets(at + bearing * (depth * 0.5 + 0.06), 0.02));
        assert!(!meets(at - bearing * (depth * 0.5 + GATE_WAY + 0.06), 0.02));
    }
}
