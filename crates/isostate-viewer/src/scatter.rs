//! Life on the land: trees, town houses, farmsteads, mines and the works
//! that firms build.
//!
//! The economy only knows shares and densities per 2 km cell. Here every
//! possible tree or house has a fixed spot and a fixed threshold, and it is
//! drawn once its cell's share or density passes that threshold. Growth is
//! therefore smooth and stable: nothing reshuffles from one year to the next,
//! towns thicken in place, and woods thin out tree by tree.
//!
//! This file decides where things stand and which of them stand this year.
//! What each looks like is a generator's business: those are plain functions
//! in the isostate-gen crate, called from here with the spot, the slot's
//! fixed numbers and the look's palette.

use bevy::math::Vec3;

pub mod towns;

use isostate_gen::bridge::{road_bridge, RoadBridge};
use isostate_gen::farm::{farmstead, Farmstead};
use isostate_gen::trees::{bush, hedge_tree, tree, Species, Tree};
use isostate_gen::works::{self, pit, works, Kind, Pit, PitKind, State, Works};
use isostate_gen::{Seed, Site};

use crate::look::Look;
use crate::meshkit::{shade, Builder};
use crate::replay::{self, Replay};
use crate::world::{self, Ground};

#[derive(Clone, Copy)]
pub struct Slot {
    pub x: f32,
    pub z: f32,
    /// the warped cell whose mix decides what grows here
    pub wcell: u32,
    /// patch-noise value at the spot, 0..255, so the slot agrees with the painted ground
    pub sub: f32,
    /// fixed thresholds and variety
    pub u: f32,
    pub v: f32,
    pub yaw: f32,
}

#[derive(Clone)]
pub struct Slots {
    pub trees: Vec<Slot>,
    pub houses: Vec<Slot>,
    pub farms: Vec<Slot>,
    pub mines: Vec<Slot>,
    /// where every firm's works stands: the middle of its yard, and the way it faces
    pub firm_pos: Vec<(f32, f32, f32)>,
    /// the largest size each works may be drawn at where it stands: less than
    /// its own size only in a cell too full to hold it
    pub firm_cap: Vec<f32>,
    /// a river runs by the works
    pub firm_water: Vec<bool>,
    /// the rung of the size each works' ground was set aside for
    firm_rung: Vec<i32>,
    /// spots on the hedges between fields, for hedgerow trees
    pub hedge: Vec<Slot>,
    /// the year the fields of each hedge spot and each farm spot are enclosed
    /// (0 if they always were), as there are no hedges round open fields and the
    /// farms stand in the village until enclosure
    pub hedge_open: Vec<i32>,
    pub farm_open: Vec<i32>,
}

fn grid_slots(r: &Replay, g: &Ground, n: usize, seed: i32, jitter: f32) -> Vec<Slot> {
    let mut out = Vec::new();
    for c in 0..r.n {
        if r.terrain[c] == 0 {
            continue;
        }
        let (cx, cy) = ((c % r.w) as f32, (c / r.w) as f32);
        for j in 0..n {
            for i in 0..n {
                let id = (c * n * n + j * n + i) as i32;
                let x = cx + (i as f32 + 0.5 + jitter * (world::hash2(id, 1, seed) - 0.5)) / n as f32;
                let z = cy + (j as f32 + 0.5 + jitter * (world::hash2(id, 2, seed) - 0.5)) / n as f32;
                // nothing on the beach or in a river
                if world::height_at(g, x, z) < 0.05 || world::water_dist(g, x, z) < 0.025 {
                    continue;
                }
                let (wc, sub) = world::class_key(g, x, z);
                out.push(Slot {
                    x,
                    z,
                    wcell: wc as u32,
                    sub,
                    u: world::hash2(id, 3, seed),
                    v: world::hash2(id, 4, seed),
                    yaw: world::hash2(id, 5, seed) * std::f32::consts::TAU,
                });
            }
        }
    }
    out
}

// ------------------------------------------------------------------ where the works stand
//
// Every works has a yard of its own, as wide and as deep as its kind needs at
// its size (`works::footprint`), and the yards of a cell must not overlap one
// another, the market place, the roads, the railway or the river. So the
// ground of a cell is handed out: each works, in the order they were founded,
// is given the first free spot (of a fixed, shuffled list) that holds the
// yard it will need at the largest size it reaches in the record. A works
// that has closed gives its ground up to those founded after. Nothing moves
// from year to year. In a live game the future is not known: when a works
// outgrows the ground set aside for it, its cell is laid out again, and its
// younger neighbours may move.
//
// A cell with more works than it has room for: a works that finds no spot is
// tried at half the size, and half again, down to the smallest; it is then
// drawn no bigger than fits (`Slots::firm_cap`). If even the smallest finds
// no spot it stands on the first spot on dry land and overlaps its neighbours.

/// The two fixed numbers of a firm's works, and the phase of its size ladder.
fn firm_seed(f: usize) -> Seed {
    Seed::new(world::hash2(f as i32, 1, 41), world::hash2(f as i32, 2, 41))
}

fn firm_phase(f: usize) -> f32 {
    world::hash2(f as i32, 3, 41)
}

fn firm_kind(r: &Replay, f: usize) -> Kind {
    Kind::of_method(&r.header.methods[r.header.firms[f].method].name)
}

/// A firm's capacity over the standard capacity of its method.
fn firm_ratio(r: &Replay, f: usize, kk: f32) -> f32 {
    kk / r.header.methods[r.header.firms[f].method].k0.max(1e-6)
}

/// The years a works stands (the first year index it has capacity, and one
/// past the last, or no end if it stands in the last year known), and the
/// most capacity it reaches.
fn firm_life(r: &Replay, f: usize) -> (usize, usize, f32) {
    let (first, len, off) = (r.firm_first[f].max(0) as usize, r.firm_len[f].max(0) as usize, r.firm_off[f].max(0) as usize);
    let (mut from, mut to, mut top) = (None, 0, 0.0f32);
    for (i, &kk) in r.firm_k[off..off + len].iter().enumerate() {
        if kk > 0.0 {
            from.get_or_insert(first + i);
            to = first + i + 1;
            top = top.max(kk);
        }
    }
    match from {
        Some(from) => (from, if to >= r.filled.min(r.ny) { usize::MAX } else { to }, top),
        None => ((r.header.firms[f].born - r.y0).max(0) as usize, usize::MAX, 0.0),
    }
}

/// The roads and railways of year `k` as points along them, by the cell each
/// point lies in, with the room to leave either side: works keep clear of them.
fn ways(r: &Replay, cen: &[(f32, f32)], k: usize) -> std::collections::HashMap<usize, Vec<(f32, f32, f32)>> {
    let mut out: std::collections::HashMap<usize, Vec<(f32, f32, f32)>> = Default::default();
    let mut lay = |a: (f32, f32), b: (f32, f32), clear: f32| {
        let len = (b.0 - a.0).hypot(b.1 - a.1);
        let n = ((len / 0.03).ceil() as usize).max(1);
        for i in 0..=n {
            let t = i as f32 / n as f32;
            let (x, z) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            let c = (z.floor() as usize).min(r.h - 1) * r.w + (x.floor() as usize).min(r.w - 1);
            out.entry(c).or_default().push((x, z, clear));
        }
    };
    for (c, m, lv) in crate::network::road_segments(r, k) {
        lay(cen[c], cen[m], if lv == 3 { 0.03 } else { 0.024 });
    }
    // the railway, as the straight lines between the middles of its cells (the line itself curves a little)
    let tr = &r.trans[k * r.n..(k + 1) * r.n];
    let has = |x: isize, y: isize| x >= 0 && y >= 0 && (x as usize) < r.w && (y as usize) < r.h && tr[y as usize * r.w + x as usize] & 4 != 0;
    for c in 0..r.n {
        if tr[c] & 4 == 0 {
            continue;
        }
        let (x, y) = ((c % r.w) as isize, (c / r.w) as isize);
        for (dx, dy) in [(1isize, 0isize), (0, 1), (1, 1), (-1, 1)] {
            if has(x + dx, y + dy) && !(dx != 0 && dy != 0 && (has(x + dx, y) || has(x, y + dy))) {
                lay(cen[c], cen[(y + dy) as usize * r.w + (x + dx) as usize], 0.05);
            }
        }
    }
    out
}

/// Ground set aside for a works: the box round its yard, and the years it stands.
struct Lot {
    x: f32,
    z: f32,
    hx: f32,
    hz: f32,
    from: usize,
    to: usize,
}

/// Lay out the works of one cell: `firms` in the order they were founded.
/// It gives back how many found no room at their own size, and how many none at all.
#[allow(clippy::too_many_arguments)]
fn lay_out(r: &Replay, g: &Ground, cen: &[(f32, f32)], ways: &[(f32, f32, f32)], cell: usize, firms: &[usize], lives: &[(usize, usize, f32)], rungs: &[i32], slots: &mut Slots) -> (usize, usize) {
    let (cx, cy) = ((cell % r.w) as f32, (cell / r.w) as f32);
    let town = r.header.towns.iter().any(|t| t.node == cell).then(|| cen[cell]);
    // the spots a yard may be centred on: a 9 x 9 grid on dry land, shuffled
    const N: usize = 9;
    let mut spots: Vec<(f32, f32, usize, f32, f32)> = Vec::new();
    for j in 0..N {
        for i in 0..N {
            let (x, z) = (cx + (i as f32 + 0.5) / N as f32, cy + (j as f32 + 0.5) / N as f32);
            if world::height_at(g, x, z) >= 0.07 {
                let (wc, sub) = world::class_key(g, x, z);
                spots.push((x, z, wc, sub, world::hash2(i as i32, j as i32, cell as i32)));
            }
        }
    }
    spots.sort_by(|a, b| a.4.total_cmp(&b.4));
    let dry = |x: f32, z: f32| world::height_at(g, x, z) >= 0.07;
    let mut lots: Vec<Lot> = Vec::new();
    let (mut squeezed, mut homeless) = (0, 0);
    for &f in firms {
        let kind = firm_kind(r, f);
        let (from, to, _) = lives[f];
        let quarter = (world::hash2(f as i32, 9, 3) * 4.0).floor() as i32;
        let own = works::rung_size(rungs[f], firm_phase(f));
        // does a yard with these half sizes fit at this spot?
        let fits = |x: f32, z: f32, hx: f32, hz: f32, lots: &[Lot]| -> bool {
            if x - hx < cx + 0.04 || x + hx > cx + 0.96 || z - hz < cy + 0.04 || z + hz > cy + 0.96 {
                return false;
            }
            if lots.iter().any(|l| (l.x - x).abs() < l.hx + hx + 0.012 && (l.z - z).abs() < l.hz + hz + 0.012 && from < l.to && l.from < to) {
                return false;
            }
            if let Some((tx, tz)) = town {
                // the market place, the church and the houses round them
                let (dx, dz) = (((tx - x).abs() - hx).max(0.0), ((tz - z).abs() - hz).max(0.0));
                if dx.hypot(dz) < 0.17 {
                    return false;
                }
            }
            if ways.iter().any(|&(wx, wz, clear)| (wx - x).abs() < hx + clear && (wz - z).abs() < hz + clear) {
                return false;
            }
            // dry land under all of it, and no river through it
            let (nx, nz) = (((2.0 * hx / 0.05).ceil() as usize).max(1), ((2.0 * hz / 0.05).ceil() as usize).max(1));
            for jz in 0..=nz {
                for ix in 0..=nx {
                    let (px, pz) = (x - hx + 2.0 * hx * ix as f32 / nx as f32, z - hz + 2.0 * hz * jz as f32 / nz as f32);
                    if !dry(px, pz) || world::water_dist(g, px, pz) < 0.045 {
                        return false;
                    }
                }
            }
            true
        };
        // the town trades take town ground first, then works ground; the rest take works ground first
        let year = from.min(r.ny - 1);
        let liked: &[u8] = if matches!(kind, Kind::HandLoom | Kind::Smithy | Kind::Millwright | Kind::Shops) { &[world::URBAN, world::WORKS] } else { &[world::WORKS] };
        let mut size = own;
        let mut found: Option<(f32, f32, i32)> = None;
        'search: loop {
            let (w, d) = works::footprint(kind, size);
            for pass in 0..=liked.len() {
                for q in [quarter, quarter + 1] {
                    let (hx, hz) = if q % 2 == 0 { (w * 0.5, d * 0.5) } else { (d * 0.5, w * 0.5) };
                    for &(x, z, wc, sub, _) in &spots {
                        if pass < liked.len() && world::pick_class(r, year, wc, sub) != liked[pass] {
                            continue;
                        }
                        if fits(x, z, hx, hz, &lots) {
                            found = Some((x, z, q));
                            break 'search;
                        }
                    }
                }
            }
            if size <= 0.4 {
                break;
            }
            size = (size * 0.5).max(0.4);
        }
        if found.is_some() && size < own {
            squeezed += 1;
        }
        let (x, z, q) = found.unwrap_or_else(|| {
            homeless += 1;
            let at = spots.get(f % spots.len().max(1)).map(|s| (s.0, s.1)).unwrap_or((cx + 0.5, cy + 0.5));
            (at.0, at.1, quarter)
        });
        let (w, d) = works::footprint(kind, size);
        let (hx, hz) = if q % 2 == 0 { (w * 0.5, d * 0.5) } else { (d * 0.5, w * 0.5) };
        // a little out of square, less for a big yard so that it keeps to its ground
        let askew = (world::hash2(f as i32, 10, 3) - 0.5) * 0.12 * (0.08 / hx.max(hz)).min(1.0);
        slots.firm_pos[f] = (x, z, q as f32 * std::f32::consts::FRAC_PI_2 + askew);
        slots.firm_cap[f] = if size < own { size } else { f32::MAX };
        slots.firm_water[f] = world::water_dist(g, x, z) < hx.hypot(hz) + 0.09;
        lots.push(Lot { x, z, hx, hz, from, to });
    }
    (squeezed, homeless)
}

/// The works of firm `f` as it is drawn in year `k`, if it stands then: the
/// size from its ladder, the state from what the year did to it.
pub fn firm_works(r: &Replay, slots: &Slots, f: usize, k: usize) -> Option<Works> {
    let kk = replay::firm_k(r, f, k);
    if kk <= 0.0 {
        return None;
    }
    let firm = &r.header.firms[f];
    let phase = firm_phase(f);
    let rung = works::rung(firm_ratio(r, f, kk), phase);
    let before = if k > 0 { replay::firm_k(r, f, k - 1) } else { kk };
    // scaffolding the first year a works stands, and the year it grows a size
    let state = if before <= 0.0 && firm.born > r.y0 {
        State::GoingUp
    } else if before > 0.0 && rung > works::rung(firm_ratio(r, f, before), phase) {
        State::Extending
    } else {
        State::Working
    };
    Some(Works {
        kind: firm_kind(r, f),
        size: works::rung_size(rung, phase).min(slots.firm_cap.get(f).copied().unwrap_or(f32::MAX)),
        founded: firm.born,
        state,
        water: slots.firm_water.get(f).copied().unwrap_or(false),
        rail: r.trans[k * r.n + firm.cell] & 4 != 0,
        seed: firm_seed(f),
    })
}

/// Where the family's flag flies on the works of firm `f` in year `k`: the
/// top of its highest gable.
pub fn firm_flag(r: &Replay, g: &Ground, slots: &Slots, f: usize, k: usize, lk: &Look) -> Option<Vec3> {
    let w = firm_works(r, slots, f, k)?;
    let &(x, z, yaw) = slots.firm_pos.get(f)?;
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    Some(works::flag_point(&Site::on(ground(g, x, z), yaw, &height), &w, lk))
}

/// Give every works its ground. A live game calls this each year as firms are
/// founded and grow: only the cells where a works is new, or has outgrown
/// the ground set aside for it, are laid out again.
pub fn place_new_firms(r: &Replay, g: &Ground, slots: &mut Slots) {
    let n = r.header.firms.len();
    let lives: Vec<(usize, usize, f32)> = (0..n).map(|f| firm_life(r, f)).collect();
    let rungs: Vec<i32> = (0..n).map(|f| works::rung(firm_ratio(r, f, lives[f].2), firm_phase(f))).collect();
    let dirty: std::collections::HashSet<usize> = (0..n).filter(|&f| slots.firm_rung.get(f).map_or(true, |&set| rungs[f] > set)).map(|f| r.header.firms[f].cell).collect();
    slots.firm_pos.resize(n, (0.0, 0.0, 0.0));
    slots.firm_cap.resize(n, f32::MAX);
    slots.firm_water.resize(n, false);
    if dirty.is_empty() {
        return;
    }
    let cen = crate::network::centres(r);
    let ways = ways(r, &cen, r.filled.clamp(1, r.ny) - 1);
    let mut by_cell: std::collections::BTreeMap<usize, Vec<usize>> = Default::default();
    for f in 0..n {
        if dirty.contains(&r.header.firms[f].cell) {
            by_cell.entry(r.header.firms[f].cell).or_default().push(f);
        }
    }
    let (mut squeezed, mut homeless) = (0, 0);
    for (cell, mut firms) in by_cell {
        firms.sort_by_key(|&f| (r.header.firms[f].born, f));
        let (a, b) = lay_out(r, g, &cen, ways.get(&cell).map_or(&[][..], |v| v), cell, &firms, &lives, &rungs, slots);
        squeezed += a;
        homeless += b;
    }
    slots.firm_rung = rungs;
    if std::env::var_os("ISOSTATE_TIMING").is_some() {
        eprintln!("works laid out in {} cells: {} of {} drawn smaller than they are for want of room, {} with no ground of their own", dirty.len(), squeezed, n, homeless);
    }
}

pub fn make_slots(r: &Replay, g: &Ground) -> Slots {
    world::timed("slots", || make_slots_inner(r, g))
}

fn make_slots_inner(r: &Replay, g: &Ground) -> Slots {
    let mut slots = Slots {
        trees: grid_slots(r, g, 6, 11, 0.9),
        houses: grid_slots(r, g, 7, 12, 0.7),
        farms: Vec::new(),
        mines: grid_slots(r, g, 2, 14, 0.7),
        firm_pos: Vec::new(),
        firm_cap: Vec::new(),
        firm_water: Vec::new(),
        firm_rung: Vec::new(),
        hedge: Vec::new(),
        hedge_open: Vec::new(),
        farm_open: Vec::new(),
    };
    (slots.hedge, slots.hedge_open) = hedge_slots(r, g).into_iter().unzip();
    slots.farms = grid_slots(r, g, 2, 13, 0.5).into_iter().map(|s| farm_spot(g, s)).collect();
    slots.farm_open = slots.farms.iter().map(|s| g.open_until[world::own_cell(g, s.x, s.z)]).collect();
    // every works gets ground in its cell, first come first served
    place_new_firms(r, g, &mut slots);
    crate::plan::maybe_write(r, g, &slots);
    crate::plan::maybe_bench(r, g, &slots, &crate::network::centres(r));
    slots
}

/// Spots on the hedges: a grid of candidates in every land cell, each moved
/// onto the nearest edge of its field. The spot keeps the class of the field it
/// started in, so a tree only grows on the hedges of farm and pasture.
fn hedge_slots(r: &Replay, g: &Ground) -> Vec<(Slot, i32)> {
    use rayon::prelude::*;
    let n = 7usize;
    (0..r.n)
        .into_par_iter()
        .flat_map_iter(|c| {
            let mut out = Vec::new();
            if r.terrain[c] == 0 {
                return out;
            }
            let (cx, cy) = ((c % r.w) as f32, (c / r.w) as f32);
            for j in 0..n {
                for i in 0..n {
                    let id = (c * n * n + j * n + i) as i32;
                    let x = cx + (i as f32 + 0.5 + 0.9 * (world::hash2(id, 1, 17) - 0.5)) / n as f32;
                    let z = cy + (j as f32 + 0.5 + 0.9 * (world::hash2(id, 2, 17) - 0.5)) / n as f32;
                    if world::height_at(g, x, z) < 0.05 {
                        continue;
                    }
                    let f = world::field_at(g, x, z);
                    if f.edge > 0.08 {
                        continue;
                    }
                    // step onto the edge, twice, as the gentle warp of the fields
                    // makes the first step fall a little short
                    let (mut hx, mut hz) = (x + f.toward.x * f.edge, z + f.toward.y * f.edge);
                    let f2 = world::field_at(g, hx - f.toward.x * 0.004, hz - f.toward.y * 0.004);
                    if f2.cut.id == f.cut.id && f2.block.i == f.block.i && f2.block.j == f.block.j {
                        hx += f2.toward.x * (f2.edge - 0.004).max(0.0);
                        hz += f2.toward.y * (f2.edge - 0.004).max(0.0);
                    }
                    if world::height_at(g, hx, hz) < 0.05 || world::water_dist(g, hx, hz) < 0.03 {
                        continue;
                    }
                    // trees stand thicker in some districts than others
                    let u = world::hash2(id, 3, 17);
                    if u > 0.05 + 0.12 * (0.5 + 0.5 * world::vnoise(hx * 0.45, hz * 0.45, 181)) {
                        continue;
                    }
                    let (wc, sub) = world::class_key(g, x, z);
                    let slot = Slot {
                        x: hx,
                        z: hz,
                        wcell: wc as u32,
                        sub,
                        u,
                        v: world::hash2(id, 4, 17),
                        yaw: world::hash2(id, 5, 17) * std::f32::consts::TAU,
                    };
                    out.push((slot, g.open_until[world::own_cell(g, x, z)]));
                }
            }
            out
        })
        .collect()
}

/// A farmstead stands just inside a hedge of its field, square to it.
fn farm_spot(g: &Ground, s: Slot) -> Slot {
    let f = world::field_at(g, s.x, s.z);
    let mut out = s;
    if f.edge > 0.045 {
        out.x += f.toward.x * (f.edge - 0.04);
        out.z += f.toward.y * (f.edge - 0.04);
    }
    // the house's back to the hedge, its front to the yard
    out.yaw = (-f.toward.y).atan2(f.toward.x) + std::f32::consts::FRAC_PI_2;
    out
}

fn ground(g: &Ground, x: f32, z: f32) -> Vec3 {
    Vec3::new(x, world::height_at(g, x, z), z)
}

// ------------------------------------------------------------------ the year's meshes

pub struct YearMeshes {
    pub trees: Builder,
    pub houses: Builder,
    pub works: Builder,
    pub smoke: Builder,
    /// farm lanes, flat on the ground
    pub lanes: Builder,
}

pub fn build_year(r: &Replay, g: &Ground, slots: &Slots, cen: &[(f32, f32)], k: usize, lk: &Look) -> YearMeshes {
    world::timed("year scatter", || build_year_inner(r, g, slots, cen, k, lk))
}

fn build_year_inner(r: &Replay, g: &Ground, slots: &Slots, cen: &[(f32, f32)], k: usize, lk: &Look) -> YearMeshes {
    let mut trees = Builder::default();
    let mut houses = Builder::default();
    let mut works_b = Builder::default();
    let mut smoke = Builder::default();

    // the generators ask this for the ground under the parts of a thing that stand apart
    let height = |x: f32, z: f32| world::height_at(g, x, z);

    // the works of the year, each in its own yard; houses, trees and farms keep clear of the yards
    let mut lanes = Builder::default();
    let mut plots: std::collections::HashMap<usize, Vec<(f32, f32, f32, f32)>> = Default::default();
    for (f, firm) in r.header.firms.iter().enumerate() {
        let Some(w) = firm_works(r, slots, f, k) else { continue };
        let p = slots.firm_pos[f];
        let (fw, fd) = works::footprint(w.kind, w.size);
        // the box round the yard as it lies on the map, and a little room
        let (sin, cos) = p.2.sin_cos();
        let (hx, hz) = ((fw * cos.abs() + fd * sin.abs()) * 0.5 + 0.02, (fw * sin.abs() + fd * cos.abs()) * 0.5 + 0.02);
        plots.entry(firm.cell).or_default().push((p.0, p.1, hx, hz));
        let site = Site::on(ground(g, p.0, p.1), p.2, &height);
        works(&mut works_b, &mut smoke, &mut lanes, &site, &w, lk);
    }
    // the railway's embankments, viaducts, cuttings, portals and bridges, and a
    // bridge wherever a road crosses a river
    works_b.append(crate::rails::structures(&crate::rails::year(r, g, cen, k), g, lk));
    for (c, m, lv) in crate::network::road_segments(r, k) {
        let (a, b) = (bevy::math::Vec2::new(cen[c].0, cen[c].1), bevy::math::Vec2::new(cen[m].0, cen[m].1));
        for (at, _, half, level) in crate::rivers::crossings(&g.rivers, &g.water, a, b) {
            road_bridge(&mut works_b, &RoadBridge { at, along: (b - a).normalize_or(bevy::math::Vec2::X), half, level, main: lv == 3 }, &height, lk);
        }
        causeway(&mut works_b, lk, g, a, b, lv == 3);
    }
    towns::landmarks(r, g, cen, k, lk, &mut houses, &mut plots);
    let near_plot = |x: f32, z: f32| -> bool {
        let c = (z.floor() as usize).min(r.h - 1) * r.w + (x.floor() as usize).min(r.w - 1);
        plots.get(&c).map(|v| v.iter().any(|&(px, pz, hx, hz)| (px - x).abs() < hx && (pz - z).abs() < hz)).unwrap_or(false)
    };
    // the town houses are placed afresh each year, so they also keep out of the rivers
    let near_plot_or_water = |x: f32, z: f32| -> bool { near_plot(x, z) || world::water_dist(g, x, z) < 0.03 };

    for s in &slots.trees {
        let cls = world::pick_class(r, k, s.wcell as usize, s.sub);
        let terr = r.terrain[s.wcell as usize];
        let (grow, scale) = match cls {
            world::WOOD => (true, 0.85 + 0.4 * s.v),
            // the odd tree out in a field; most stand in the hedges
            world::FARM | world::PASTURE => (s.u < 0.012, 0.75 + 0.3 * s.v),
            world::HEATH => {
                // gorse and broom: low bushes, and the odd tree
                if s.u < 0.16 && !near_plot(s.x, s.z) {
                    bush(&mut trees, &Site::level(ground(g, s.x, s.z), 0.0), s.v, s.yaw, lk);
                }
                (s.u < if terr >= 3 { 0.02 } else { 0.035 }, 0.6 + 0.3 * s.v)
            }
            _ => (false, 1.0),
        };
        if !grow || near_plot(s.x, s.z) {
            continue;
        }
        let p = ground(g, s.x, s.z);
        let conifer = terr >= 2 && s.v > 0.35 || s.v > 0.88;
        let species = if conifer { Species::Conifer } else { Species::Broadleaf };
        tree(&mut trees, &Site::level(p, 0.0), &Tree { species, size: scale, tint: s.u, turn: s.yaw }, lk);
    }

    towns::houses(r, g, slots, cen, k, lk, &near_plot_or_water, &mut houses, &mut smoke);

    // hedgerow trees, once the fields are enclosed, and never in a road
    let year = r.y0 + k as i32;
    let roads = RoadIndex::new(r, cen, k);
    for (s, &open) in slots.hedge.iter().zip(&slots.hedge_open) {
        if year < open {
            continue;
        }
        let cls = world::pick_class(r, k, s.wcell as usize, s.sub);
        if !(cls == world::FARM || cls == world::PASTURE) || near_plot(s.x, s.z) || roads.nearest(s.x, s.z, 0.03).is_some() {
            continue;
        }
        let p = ground(g, s.x, s.z);
        hedge_tree(&mut trees, &Site::level(p, 0.0), 0.55 + 0.3 * s.v, s.u * 7.3, s.yaw, lk);
    }

    // farmsteads among the fields, each with a lane to the nearest road
    for (s, &open) in slots.farms.iter().zip(&slots.farm_open) {
        let c = s.wcell as usize;
        let off = k * r.n + c;
        let main = r.mix_a[off];
        if year < open || !(main == world::FARM || main == world::PASTURE) || s.u > 0.32 || near_plot(s.x, s.z) {
            continue;
        }
        let cls = world::pick_class(r, k, c, s.sub);
        if !(cls == world::FARM || cls == world::PASTURE) || roads.nearest(s.x, s.z, 0.075).is_some() {
            continue;
        }
        // stone and slate in the hills
        let upland = r.terrain[c] >= 2;
        let site = Site::on(ground(g, s.x, s.z), s.yaw, &height);
        farmstead(&mut houses, &mut trees, &site, &Farmstead { arable: main == world::FARM, upland, seed: Seed::new(s.u, s.v) }, lk);
        let q = bevy::math::Quat::from_rotation_y(s.yaw);
        let gate = Vec3::new(s.x, 0.0, s.z) + q * Vec3::new(0.0, 0.0, -0.05);
        if let Some(to) = roads.nearest(gate.x, gate.z, 0.55) {
            lane(&mut lanes, lk, g, (gate.x, gate.z), to, s.v);
        }
    }

    for s in &slots.mines {
        let c = s.wcell as usize;
        let cls = world::pick_class(r, k, c, s.sub);
        let kind = match cls {
            world::COALMINE => PitKind::Coal,
            world::IRONMINE => PitKind::Ironstone,
            world::QUARRY => PitKind::Stone,
            world::CLAYPIT => PitKind::Clay,
            _ => continue,
        };
        if s.u > 0.7 || near_plot(s.x, s.z) {
            continue;
        }
        pit(&mut works_b, &Site::level(ground(g, s.x, s.z), s.yaw), &Pit { kind, v: s.v }, lk);
    }

    YearMeshes { trees, houses, works: works_b, smoke, lanes }
}

/// A low causeway of stone wherever a road runs over water (the drowned mouth
/// of a river, a creek), so the road does not lie on the sea.
fn causeway(b: &mut Builder, lk: &Look, g: &Ground, a: bevy::math::Vec2, c: bevy::math::Vec2, main: bool) {
    let len = a.distance(c);
    let n = ((len / 0.02).ceil() as usize).max(2);
    let wet = |t: f32| {
        let p = a + (c - a) * t;
        world::height_at(g, p.x, p.y) < 0.004
    };
    let stone = *lk.bridges.causeway;
    let width = if main { 0.046 } else { 0.034 };
    let dir = (c - a).normalize_or(bevy::math::Vec2::X);
    let side = Vec3::new(-dir.y, 0.0, dir.x) * (width * 0.5);
    let mut i = 0;
    while i < n {
        if !wet(i as f32 / n as f32) {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < n && wet(j as f32 / n as f32) {
            j += 1;
        }
        // from one dry end to the other, a little past each
        let (t0, t1) = (((i as f32 - 1.0) / n as f32).max(0.0), ((j as f32 + 0.5) / n as f32).min(1.0));
        let (p0, p1) = (a + (c - a) * t0, a + (c - a) * t1);
        let top = 0.016;
        let (q0, q1) = (Vec3::new(p0.x, top, p0.y), Vec3::new(p1.x, top, p1.y));
        let foot = Vec3::Y * (top + 0.03);
        b.quad(q0 + side, q1 + side, q1 - side, q0 - side, shade(stone, 1.06));
        b.quad(q0 + side - foot, q1 + side - foot, q1 + side, q0 + side, stone);
        b.quad(q1 - side - foot, q0 - side - foot, q0 - side, q1 - side, shade(stone, 0.85));
        i = j;
    }
}

/// The year's road segments, bucketed by cell, for finding the nearest road.
struct RoadIndex {
    w: usize,
    h: usize,
    cells: std::collections::HashMap<usize, Vec<((f32, f32), (f32, f32))>>,
}

impl RoadIndex {
    fn new(r: &Replay, cen: &[(f32, f32)], k: usize) -> RoadIndex {
        let mut cells: std::collections::HashMap<usize, Vec<((f32, f32), (f32, f32))>> = Default::default();
        for (c, m, _) in crate::network::road_segments(r, k) {
            let seg = (cen[c], cen[m]);
            cells.entry(c).or_default().push(seg);
            cells.entry(m).or_default().push(seg);
        }
        RoadIndex { w: r.w, h: r.h, cells }
    }

    /// The nearest point on a road within `reach`, if any.
    fn nearest(&self, x: f32, z: f32, reach: f32) -> Option<(f32, f32)> {
        let (cx, cy) = (x.floor() as isize, z.floor() as isize);
        let mut best: Option<(f32, (f32, f32))> = None;
        for dy in -1..=1isize {
            for dx in -1..=1isize {
                let (xx, yy) = (cx + dx, cy + dy);
                if xx < 0 || yy < 0 || xx as usize >= self.w || yy as usize >= self.h {
                    continue;
                }
                let Some(v) = self.cells.get(&(yy as usize * self.w + xx as usize)) else { continue };
                for &(a, b) in v {
                    let (abx, abz) = (b.0 - a.0, b.1 - a.1);
                    let t = (((x - a.0) * abx + (z - a.1) * abz) / (abx * abx + abz * abz).max(1e-9)).clamp(0.0, 1.0);
                    let (px, pz) = (a.0 + abx * t, a.1 + abz * t);
                    let d = ((px - x).powi(2) + (pz - z).powi(2)).sqrt();
                    if d < reach && best.map_or(true, |b| d < b.0) {
                        best = Some((d, (px, pz)));
                    }
                }
            }
        }
        best.map(|b| b.1)
    }
}

/// A farm lane: a pale track from the yard gate to the road, bending a little.
fn lane(b: &mut Builder, lk: &Look, g: &Ground, from: (f32, f32), to: (f32, f32), v: f32) {
    let (dx, dz) = (to.0 - from.0, to.1 - from.1);
    let len = (dx * dx + dz * dz).sqrt();
    if len < 0.03 {
        return;
    }
    // a lane does not ford a river
    if !crate::rivers::crossings(&g.rivers, &g.water, bevy::math::Vec2::new(from.0, from.1), bevy::math::Vec2::new(to.0, to.1)).is_empty() {
        return;
    }
    let bend = (v - 0.5) * 0.3 * len;
    let (mx, mz) = (from.0 + dx * 0.5 - dz / len * bend, from.1 + dz * 0.5 + dx / len * bend);
    let n = ((len / 0.03).ceil() as usize).max(2);
    let pts: Vec<Vec3> = (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            let (x, z) = (u * u * from.0 + 2.0 * u * t * mx + t * t * to.0, u * u * from.1 + 2.0 * u * t * mz + t * t * to.1);
            Vec3::new(x, world::height_at(g, x, z).max(0.0) + 0.006, z)
        })
        .collect();
    b.ribbon(&pts, 0.011, *lk.ground.farm_lane);
}
