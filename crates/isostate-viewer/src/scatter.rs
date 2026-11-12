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
use isostate_gen::houses::Detail;
use isostate_gen::trees::{bush, hedge_tree, leaf_group, tree, Greenery, Species, Tree, LEAF_GROUPS};
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
    /// where the spot lies in the field system (`world::place_at`), for asking
    /// the year's land what it stands on
    pub place: world::Place,
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
    /// the churchyards the works' ground was taken clear of (`towns::Churchyard::mark`, together)
    churchyards: u64,
    /// spots on the hedges between fields, for hedgerow trees
    pub hedge: Vec<Slot>,
    /// the year the fields of each hedge spot and each farm spot are enclosed
    /// (0 if they always were), as there are no hedges round open fields and the
    /// farms stand in the village until enclosure
    pub hedge_open: Vec<i32>,
    pub farm_open: Vec<i32>,
}

/// A jittered grid of n by n spots in every land cell. `placed` finds each
/// spot's place in the field system too, for the things that go by the open
/// land's classes (trees, farms); houses and pits go by the cell rule alone.
fn grid_slots(r: &Replay, g: &Ground, n: usize, seed: i32, jitter: f32, placed: bool) -> Vec<Slot> {
    use rayon::prelude::*;
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
                        place: if placed { world::place_at(g, x, z) } else { world::Place::default() },
                        u: world::hash2(id, 3, seed),
                        v: world::hash2(id, 4, seed),
                        yaw: world::hash2(id, 5, seed) * std::f32::consts::TAU,
                    });
                }
            }
            out
        })
        .collect()
}

// ------------------------------------------------------------------ where the works stand
//
// Every works has a yard of its own, as wide and as deep as its kind needs at
// its size (`works::footprint`), and the yards of a cell must not overlap one
// another, the market place, a churchyard (in any year the church stands
// there: `towns::churchyards`), the roads, the railway or the river. So the
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
// drawn no bigger than fits (`Slots::firm_cap`). If the smallest finds no
// spot either (a cell that is nearly all shore, river or road), it is tried
// on a finer grid with less room to the water and none to the roads; then
// anywhere above the tide that no other yard has taken, wall to wall with
// its neighbours; and only if that fails too does it stand on the cell's
// highest land and overlap them. ISOSTATE_TIMING says how many of each
// there were, and ISOSTATE_PLOTS names the cells.

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

/// A point of a road or a railway that works keep clear of: where it is, the
/// room to leave either side of it, and the room to leave even where ground
/// is short (none for a road; a railway's own bed, which no yard can stand on).
type Way = (f32, f32, f32, f32);

/// The roads and railways of year `k` as points along them, by the cell each
/// point lies in: works keep clear of them. The railway is taken where it
/// runs (`rails.rs`), not as straight lines between the middles of its cells.
fn ways(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize) -> std::collections::HashMap<usize, Vec<Way>> {
    let mut out: std::collections::HashMap<usize, Vec<Way>> = Default::default();
    let mut lay = |a: (f32, f32), b: (f32, f32), clear: f32| {
        let len = (b.0 - a.0).hypot(b.1 - a.1);
        let n = ((len / 0.03).ceil() as usize).max(1);
        for i in 0..=n {
            let t = i as f32 / n as f32;
            let (x, z) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            let c = (z.floor() as usize).min(r.h - 1) * r.w + (x.floor() as usize).min(r.w - 1);
            out.entry(c).or_default().push((x, z, clear, 0.0));
        }
    };
    for (c, m, lv) in crate::network::road_segments(r, k) {
        lay(cen[c], cen[m], if lv == 3 { 0.03 } else { 0.024 });
    }
    // the railway where it really runs, with room for its banks: every point of every line that
    // is not under the ground, told to the cell it lies in and to a neighbour it lies close by
    let rails = crate::rails::year(r, g, cen, k);
    let mut tell = |x: f32, z: f32, clear: f32, at_least: f32| {
        let cell = |x: f32, z: f32| (z.floor().max(0.0) as usize).min(r.h - 1) * r.w + (x.floor().max(0.0) as usize).min(r.w - 1);
        let mut told: Vec<usize> = Vec::with_capacity(3);
        for (dx, dz) in [(0.0f32, 0.0f32), (0.12, 0.0), (-0.12, 0.0), (0.0, 0.12), (0.0, -0.12)] {
            let c = cell(x + dx, z + dz);
            if !told.contains(&c) {
                told.push(c);
                out.entry(c).or_default().push((x, z, clear, at_least));
            }
        }
    };
    for line in &rails.lines {
        for (i, p) in line.pts.iter().enumerate() {
            if line.kind[i] != crate::rails::Kind::Tunnel {
                tell(p.x, p.z, 0.05 + (1.2 * (p.y - line.ground[i])).clamp(0.0, 0.12), 0.03);
            }
        }
    }
    // and the ground of its stations, engine houses and sheds
    for (at, clear) in rails.grounds() {
        tell(at.x, at.y, clear, 0.0);
    }
    out
}

/// How much room a yard is given when a spot is tried for it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Room {
    /// well back from the shore, the rivers, the roads, the railway and the cell's edge
    Proper,
    /// nearer the water, and with no care for the roads (but off the railway's bed)
    Tight,
    /// anywhere above the tide that no other yard has taken and no line runs over, close up to its neighbours
    Any,
}

impl Room {
    /// The lowest land a yard may stand on.
    fn dry(self) -> f32 {
        match self {
            Room::Proper => 0.07,
            Room::Tight => 0.045,
            Room::Any => 0.015,
        }
    }

    /// How far a yard keeps from a river.
    fn shore(self) -> f32 {
        match self {
            Room::Proper => 0.045,
            Room::Tight => 0.02,
            Room::Any => 0.008,
        }
    }
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
fn lay_out(r: &Replay, g: &Ground, cen: &[(f32, f32)], ways: &[Way], yards: &[towns::Churchyard], cell: usize, firms: &[usize], lives: &[(usize, usize, f32)], rungs: &[i32], slots: &mut Slots) -> (usize, usize) {
    let (cx, cy) = ((cell % r.w) as f32, (cell / r.w) as f32);
    // the churchyards that reach into this cell, or near it
    let yards: Vec<&towns::Churchyard> = yards.iter().filter(|y| y.meets_box(cx + 0.5, cy + 0.5, 0.55, 0.55)).collect();
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
    // where the land is nearly all shore, river or road, the smallest works make do with less:
    // a finer grid of spots, highest first, nearer the water and with no care for the roads
    // (`Room::Tight`), and at the last anywhere above the tide (`Room::Any`)
    const FINE: usize = 18;
    let mut tight: Vec<(f32, f32, f32)> = Vec::new();
    for j in 0..FINE {
        for i in 0..FINE {
            let (x, z) = (cx + (i as f32 + 0.5) / FINE as f32, cy + (j as f32 + 0.5) / FINE as f32);
            let h = world::height_at(g, x, z);
            if h >= Room::Any.dry() {
                tight.push((x, z, h));
            }
        }
    }
    tight.sort_by(|a, b| b.2.total_cmp(&a.2));
    let mut lots: Vec<Lot> = Vec::new();
    let (mut squeezed, mut homeless) = (0, 0);
    for &f in firms {
        let kind = firm_kind(r, f);
        let (from, to, _) = lives[f];
        let quarter = (world::hash2(f as i32, 9, 3) * 4.0).floor() as i32;
        let own = works::rung_size(rungs[f], firm_phase(f));
        // does a yard with these half sizes fit at this spot?
        let fits = |x: f32, z: f32, hx: f32, hz: f32, lots: &[Lot], room: Room| -> bool {
            let strict = room == Room::Proper;
            let edge = if strict { 0.04 } else { 0.01 };
            if x - hx < cx + edge || x + hx > cx + 1.0 - edge || z - hz < cy + edge || z + hz > cy + 1.0 - edge {
                return false;
            }
            let apart = if room == Room::Any { 0.004 } else { 0.012 };
            if lots.iter().any(|l| (l.x - x).abs() < l.hx + hx + apart && (l.z - z).abs() < l.hz + hz + apart && from < l.to && l.from < to) {
                return false;
            }
            if let Some((tx, tz)) = town {
                // the market place, the church and the houses round them
                let (dx, dz) = (((tx - x).abs() - hx).max(0.0), ((tz - z).abs() - hz).max(0.0));
                if dx.hypot(dz) < 0.17 {
                    return false;
                }
            }
            // a churchyard and the way to its gate, in any year the two both stand: a works
            // keeps as far from the wall as a tree does, or a wall's width where room is short
            let wall = if strict { 0.03 } else { 0.008 };
            if yards.iter().any(|y| from < y.to && y.from < to && y.meets_box(x, z, hx + wall, hz + wall)) {
                return false;
            }
            // the roads and the railway; where room is short, only the railway's own bed
            if ways.iter().any(|&(wx, wz, clear, at_least)| {
                let clear = if strict { clear } else { at_least };
                clear > 0.0 && (wx - x).abs() < hx + clear && (wz - z).abs() < hz + clear
            }) {
                return false;
            }
            let (dry, shore) = (room.dry(), room.shore());
            // dry land under all of it, and no river through it
            let (nx, nz) = (((2.0 * hx / 0.05).ceil() as usize).max(1), ((2.0 * hz / 0.05).ceil() as usize).max(1));
            for jz in 0..=nz {
                for ix in 0..=nx {
                    let (px, pz) = (x - hx + 2.0 * hx * ix as f32 / nx as f32, z - hz + 2.0 * hz * jz as f32 / nz as f32);
                    if world::height_at(g, px, pz) < dry || world::water_dist(g, px, pz) < shore {
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
                        if fits(x, z, hx, hz, &lots, Room::Proper) {
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
        if found.is_none() {
            let (w, d) = works::footprint(kind, size);
            'tight: for room in [Room::Tight, Room::Any] {
                for q in [quarter, quarter + 1] {
                    let (hx, hz) = if q % 2 == 0 { (w * 0.5, d * 0.5) } else { (d * 0.5, w * 0.5) };
                    for &(x, z, _) in &tight {
                        if fits(x, z, hx, hz, &lots, room) {
                            found = Some((x, z, q));
                            break 'tight;
                        }
                    }
                }
            }
        }
        if found.is_some() && size < own {
            squeezed += 1;
        }
        let (w, d) = works::footprint(kind, size);
        let (x, z, q) = found.unwrap_or_else(|| {
            homeless += 1;
            // no ground at all: it overlaps its neighbours, as little as may be. Of the spots
            // where its yard would stand on land (or of them all, if there is none), the one
            // furthest from the yards already there; the higher, of two as far
            let (hx, hz) = if quarter % 2 == 0 { (w * 0.5, d * 0.5) } else { (d * 0.5, w * 0.5) };
            let ashore = |s: &&(f32, f32, f32)| [(-hx, -hz), (hx, -hz), (hx, hz), (-hx, hz)].iter().all(|&(dx, dz)| world::height_at(g, s.0 + dx, s.1 + dz) >= 0.004) && !yards.iter().any(|y| from < y.to && y.from < to && y.meets_box(s.0, s.1, hx, hz));
            let off_the_line = |s: &&(f32, f32, f32)| !ways.iter().any(|&(wx, wz, _, at_least)| at_least > 0.0 && (wx - s.0).abs() < hx + at_least && (wz - s.1).abs() < hz + at_least);
            let apart = |s: &(f32, f32, f32)| lots.iter().filter(|l| from < l.to && l.from < to).map(|l| (l.x - s.0).hypot(l.z - s.1)).fold(f32::MAX, f32::min);
            let furthest = |spots: Vec<&(f32, f32, f32)>| spots.into_iter().rev().max_by(|a, b| apart(a).total_cmp(&apart(b))).map(|s| (s.0, s.1));
            let at = furthest(tight.iter().filter(ashore).filter(off_the_line).collect()).or_else(|| furthest(tight.iter().filter(ashore).collect())).or_else(|| furthest(tight.iter().collect())).unwrap_or((cx + 0.5, cy + 0.5));
            (at.0, at.1, quarter)
        });
        let (hx, hz) = if q % 2 == 0 { (w * 0.5, d * 0.5) } else { (d * 0.5, w * 0.5) };
        // a little out of square, less for a big yard so that it keeps to its ground
        let askew = (world::hash2(f as i32, 10, 3) - 0.5) * 0.12 * (0.08 / hx.max(hz)).min(1.0);
        slots.firm_pos[f] = (x, z, q as f32 * std::f32::consts::FRAC_PI_2 + askew);
        slots.firm_cap[f] = if size < own { size } else { f32::MAX };
        slots.firm_water[f] = world::water_dist(g, x, z) < hx.hypot(hz) + 0.09;
        lots.push(Lot { x, z, hx, hz, from, to });
    }
    if homeless > 0 && std::env::var_os("ISOSTATE_PLOTS").is_some() {
        let most = (0..r.ny).map(|k| lots.iter().filter(|l| l.from <= k && k < l.to).count()).max().unwrap_or(0);
        eprintln!("cell {cell}: {} works over the years, {most} at once at most, {} spots, town {}, {} points of road and rail, {homeless} with no ground", firms.len(), spots.len(), town.is_some(), ways.len());
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

/// The flag points already found, by firm, each with the size, the state and
/// the place it was found for. Finding one builds the whole works, so a year
/// asks only for those that changed.
#[derive(Default)]
pub struct Flags(std::collections::HashMap<usize, ((u32, State, [u32; 3]), Vec3)>);

/// Where the family's flag flies on the works of firm `f` in year `k`: the
/// top of its highest gable.
pub fn firm_flag(r: &Replay, g: &Ground, slots: &Slots, f: usize, k: usize, lk: &Look, known: &mut Flags) -> Option<Vec3> {
    let w = firm_works(r, slots, f, k)?;
    let &(x, z, yaw) = slots.firm_pos.get(f)?;
    let key = (w.size.to_bits(), w.state, [x.to_bits(), z.to_bits(), yaw.to_bits()]);
    if let Some(&(_, at)) = known.0.get(&f).filter(|(was, _)| *was == key) {
        return Some(at);
    }
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    let at = works::flag_point(&Site::on(ground(g, x, z), yaw, &height), &w, lk);
    known.0.insert(f, (key, at));
    Some(at)
}

/// Give every works its ground. A live game calls this each year as firms are
/// founded and grow: only the cells where a works is new, or has outgrown
/// the ground set aside for it, are laid out again.
pub fn place_new_firms(r: &Replay, g: &Ground, slots: &mut Slots) {
    let n = r.header.firms.len();
    let lives: Vec<(usize, usize, f32)> = (0..n).map(|f| firm_life(r, f)).collect();
    let rungs: Vec<i32> = (0..n).map(|f| works::rung(firm_ratio(r, f, lives[f].2), firm_phase(f))).collect();
    let mut dirty: std::collections::HashSet<usize> = (0..n).filter(|&f| slots.firm_rung.get(f).map_or(true, |&set| rungs[f] > set)).map(|f| r.header.firms[f].cell).collect();
    slots.firm_pos.resize(n, (0.0, 0.0, 0.0));
    slots.firm_cap.resize(n, f32::MAX);
    slots.firm_water.resize(n, false);
    let cen = crate::network::centres(r);
    // the churchyards of every year so far. When one is new (a town founded, a church rebuilt
    // bigger, or moved by a road or the railway), the cells it reaches are laid out again,
    // so that a works standing where the church has come to gives way to it
    let yards = world::timed("churchyards", || towns::churchyards(r, g, &cen));
    let marks = yards.iter().fold(0u64, |all, y| all ^ y.mark());
    if marks != slots.churchyards {
        slots.churchyards = marks;
        for y in &yards {
            let reach = |c: usize| y.meets_box((c % r.w) as f32 + 0.5, (c / r.w) as f32 + 0.5, 0.5, 0.5);
            dirty.extend(r.header.firms.iter().map(|f| f.cell).filter(|&c| reach(c)));
        }
    }
    if dirty.is_empty() {
        return;
    }
    let ways = ways(r, g, &cen, r.filled.clamp(1, r.ny) - 1);
    let mut by_cell: std::collections::BTreeMap<usize, Vec<usize>> = Default::default();
    for f in 0..n {
        if dirty.contains(&r.header.firms[f].cell) {
            by_cell.entry(r.header.firms[f].cell).or_default().push(f);
        }
    }
    let (mut squeezed, mut homeless) = (0, 0);
    for (cell, mut firms) in by_cell {
        firms.sort_by_key(|&f| (r.header.firms[f].born, f));
        let (a, b) = lay_out(r, g, &cen, ways.get(&cell).map_or(&[][..], |v| v), &yards, cell, &firms, &lives, &rungs, slots);
        squeezed += a;
        homeless += b;
    }
    if std::env::var_os("ISOSTATE_TIMING").is_some() {
        // the works whose ground, as it was set aside, meets a churchyard or the way to its gate in a year both stand
        let in_churchyard = (0..n)
            .filter(|&f| {
                let ((x, z, yaw), (from, to, _)) = (slots.firm_pos[f], lives[f]);
                let (w, d) = works::footprint(firm_kind(r, f), works::rung_size(rungs[f], firm_phase(f)).min(slots.firm_cap[f]));
                let (hx, hz) = if (yaw / std::f32::consts::FRAC_PI_2).round() as i32 % 2 == 0 { (w * 0.5, d * 0.5) } else { (d * 0.5, w * 0.5) };
                yards.iter().any(|y| from < y.to && y.from < to && y.meets_box(x, z, hx, hz))
            })
            .count();
        if std::env::var_os("ISOSTATE_PLOTS").is_some() {
            for y in &yards {
                eprintln!("the church of town {} stands at {:.2}, {:.2} from {}{}", y.town, y.at.x, y.at.y, r.y0 + y.from as i32, if y.to == usize::MAX { String::new() } else { format!(" to {}", r.y0 + y.to as i32) });
            }
        }
        eprintln!("works laid out in {} cells: {} of {} drawn smaller than they are for want of room, {} with no ground of their own, {} on the ground of a church ({} churchyards over the years)", dirty.len(), squeezed, n, homeless, in_churchyard, yards.len());
    }
    slots.firm_rung = rungs;
}

pub fn make_slots(r: &Replay, g: &Ground) -> Slots {
    world::timed("slots", || make_slots_inner(r, g))
}

fn make_slots_inner(r: &Replay, g: &Ground) -> Slots {
    let mut slots = Slots {
        trees: grid_slots(r, g, 7, 11, 0.9, true),
        houses: grid_slots(r, g, 7, 12, 0.7, false),
        farms: Vec::new(),
        mines: grid_slots(r, g, 2, 14, 0.7, false),
        firm_pos: Vec::new(),
        firm_cap: Vec::new(),
        firm_water: Vec::new(),
        firm_rung: Vec::new(),
        churchyards: 0,
        hedge: Vec::new(),
        hedge_open: Vec::new(),
        farm_open: Vec::new(),
    };
    (slots.hedge, slots.hedge_open) = hedge_slots(r, g).into_iter().unzip();
    slots.farms = grid_slots(r, g, 2, 13, 0.5, true).into_iter().map(|s| farm_spot(g, s)).collect();
    slots.farm_open = slots.farms.iter().map(|s| g.land.open_until[s.place.parcel as usize]).collect();
    // every works gets ground in its cell, first come first served
    place_new_firms(r, g, &mut slots);
    crate::plan::maybe_areas(r, g);
    crate::rails::maybe_check(r, g);
    crate::plan::maybe_write(r, g, &slots);
    // (task A) ISOSTATE_TOWNS=years: every year's towns laid out in turn and counted
    towns::maybe_years(r, g, &slots);
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
                        place: world::place_at(g, x, z),
                        u,
                        v: world::hash2(id, 4, 17),
                        yaw: world::hash2(id, 5, 17) * std::f32::consts::TAU,
                    };
                    out.push((slot, g.land.open_until[slot.place.parcel as usize]));
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

use crate::fields::smoothstep as fields_smooth;

fn ground(g: &Ground, x: f32, z: f32) -> Vec3 {
    Vec3::new(x, world::height_at(g, x, z), z)
}

// ------------------------------------------------------------------ how much of each house is drawn
//
// A big town has ten thousand houses, and a house drawn in full (windows let
// into the wall, a yard with its wall and its privy) is some hundreds of
// triangles. So the houses of a year are built at three levels
// (`houses::Detail`), chosen by where the camera was looking and from how
// far when the year's scenery was begun: in full within sight of a camera
// that is close, with openings, stacks and back wings within sight of one
// at the distance the game is played from, and as shells and roofs
// elsewhere. When the camera has moved or zoomed so far that what it sees is
// no longer covered, the year's scenery is built again (`stale`), off the
// main thread, as it is for a new year; nothing but the houses and their
// smoke differs by the camera, so nothing else is put on screen again
// (`rebuild_year`).

/// Where the camera was looking and from how far when a year's scenery was
/// begun, which settles how much of each house is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lod {
    /// where the camera looks, on the map
    pub focus: (f32, f32),
    /// how far it is from there
    pub dist: f32,
    /// every house at this level, wherever the camera is
    all: Option<Detail>,
}

impl Lod {
    /// A camera this close or closer has the houses it sees drawn in full.
    pub const NEAR: f32 = 4.0;
    /// One this close or closer has them drawn with their openings and stacks.
    pub const PLAY: f32 = 40.0;

    pub fn of_camera(focus: (f32, f32), dist: f32) -> Lod {
        Lod { focus, dist, all: None }
    }

    /// Every house of the island at one level.
    pub fn all(detail: Detail) -> Lod {
        Lod { focus: (0.0, 0.0), dist: 0.0, all: Some(detail) }
    }

    /// The level the houses in the middle of the view are drawn at from a distance.
    fn band(dist: f32) -> Detail {
        if dist <= Lod::NEAR {
            Detail::Near
        } else if dist <= Lod::PLAY {
            Detail::Play
        } else {
            Detail::Far
        }
    }

    /// How far from the middle of the view the houses must be at the level
    /// its distance asks for, and how far out they are built at it, which
    /// leaves room to pan and to draw back before they must be built again.
    /// (At the game's tilt the ground at the top corners of the picture is
    /// a third as far again from the middle as the camera is. What stands
    /// out there is further from the camera, so smaller, and the lens
    /// softens it: the level below serves.)
    fn sight(dist: f32) -> f32 {
        dist + 0.25
    }

    fn reach(dist: f32) -> f32 {
        1.6 * dist + 1.2
    }

    /// The level a house at a point of the map is drawn at.
    pub fn at(&self, x: f32, z: f32) -> Detail {
        if let Some(detail) = self.all {
            return detail;
        }
        let band = Lod::band(self.dist);
        let away = (x - self.focus.0).hypot(z - self.focus.1);
        if band == Detail::Far || away <= Lod::reach(self.dist) {
            band
        } else if band == Detail::Near && away <= Lod::reach(Lod::NEAR * 2.0) {
            // beyond what a close camera sees, the middle distance: the haze and the lens soften it
            Detail::Play
        } else {
            Detail::Far
        }
    }

    /// Whether scenery built for this camera will not do for the camera as
    /// it is `now`: some house it sees is drawn with less than it should
    /// be, or it has drawn far enough back that all are drawn with more.
    pub fn stale(&self, now: &Lod) -> bool {
        if self.all.is_some() || now.all.is_some() {
            return self.all != now.all;
        }
        let want = Lod::band(now.dist);
        let sight = Lod::sight(now.dist);
        let short = (0..8).any(|i| {
            let a = i as f32 * std::f32::consts::FRAC_PI_4;
            self.at(now.focus.0 + a.cos() * sight, now.focus.1 + a.sin() * sight) < want
        });
        short || self.at(now.focus.0, now.focus.1) < want || (Lod::band(self.dist) > want && now.dist > self.dist * 1.3)
    }
}

// ------------------------------------------------------------------ the year's meshes

pub struct YearMeshes {
    /// the trees and bushes, the broadleaves in the groups that come into
    /// leaf and shed together (each group is three meshes: what is green all
    /// year, the leaves, and the bare limbs shown when the leaves are down)
    pub trees: [Greenery; LEAF_GROUPS],
    pub houses: Builder,
    pub works: Builder,
    pub smoke: Builder,
    /// what lies flat on the ground: farm lanes, and the yards of the works
    pub lanes: Builder,
}

pub fn build_year(r: &Replay, g: &Ground, slots: &Slots, cen: &[(f32, f32)], k: usize, lk: &Look, lod: &Lod) -> YearMeshes {
    build_year_on(r, g, slots, cen, k, lk, lod, &world::year_land(r, g, k))
}

/// (task C, the land) The year's scenery on a year's land already worked out
/// (`world::year_land`), so that one reckoning of it serves the ground's
/// textures and the scenery.
#[allow(clippy::too_many_arguments)]
pub fn build_year_on(r: &Replay, g: &Ground, slots: &Slots, cen: &[(f32, f32)], k: usize, lk: &Look, lod: &Lod, land: &world::YearLand) -> YearMeshes {
    world::timed("year scatter", || build_year_inner(r, g, slots, cen, k, lk, lod, land))
}

#[allow(clippy::too_many_arguments)]
fn build_year_inner(r: &Replay, g: &Ground, slots: &Slots, cen: &[(f32, f32)], k: usize, lk: &Look, lod: &Lod, land: &world::YearLand) -> YearMeshes {
    let mut trees: [Greenery; LEAF_GROUPS] = Default::default();
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
    // the railway's banks, viaducts, portals and bridges and what stands by it, and a
    // bridge wherever a road crosses a river
    let (rail_works, rail_smoke) = crate::rails::structures(&crate::rails::year(r, g, cen, k), g, lk);
    works_b.append(rail_works);
    smoke.append(rail_smoke);
    for (c, m, lv) in crate::network::road_segments(r, k) {
        let (a, b) = (bevy::math::Vec2::new(cen[c].0, cen[c].1), bevy::math::Vec2::new(cen[m].0, cen[m].1));
        let along = (b - a).normalize_or(bevy::math::Vec2::X);
        for (at, river, half, level) in crate::rivers::crossings(&g.rivers, &g.water, a, b) {
            // a road that crosses aslant has further to go from bank to bank (up to three times as far)
            let half = half / along.perp_dot(river).abs().max(0.34);
            road_bridge(&mut works_b, &RoadBridge { at, along, half, level, main: lv == 3 }, &height, lk);
        }
        causeway(&mut works_b, lk, g, a, b, lv == 3);
    }
    let parishes = towns::landmarks(r, g, cen, k, lk, &mut houses, &mut plots);
    // near a works or a market place, in a churchyard, or on the railway or its banks
    let rail_year = crate::rails::year(r, g, cen, k);
    let near_plot = |x: f32, z: f32| -> bool {
        let c = (z.floor() as usize).min(r.h - 1) * r.w + (x.floor() as usize).min(r.w - 1);
        plots.get(&c).map(|v| v.iter().any(|&(px, pz, hx, hz)| (px - x).abs() < hx && (pz - z).abs() < hz)).unwrap_or(false)
            || parishes.iter().any(|p| p.holds(bevy::math::Vec2::new(x, z), 0.03))
            || rail_year.near(x, z, 0.016)
    };
    // the town houses are placed afresh each year, so they also keep out of the rivers
    let near_plot_or_water = |x: f32, z: f32| -> bool { near_plot(x, z) || world::water_dist(g, x, z) < 0.03 };

    // Woods with a shape of their own. A wood is thick and tall in its depths
    // and thinner and lower toward its edge (`YearLand::wooded`), with glades
    // and rides the trees keep out of (`world::glade`, which the ground paints
    // too); the country next to a wood carries clumps and lone trees that thin
    // out away from it; and the ground just outside a wood or a heath is scrub.
    // Trees stand in stands, taller in one and lower in the next, and the
    // conifers gather on high and poor ground, the broadleaves on the low.
    for s in &slots.trees {
        let (cls, margin) = land.at(s.place);
        if !(world::FARM..=world::HEATH).contains(&cls) {
            continue;
        }
        // the tree line: fewer and lower trees and bushes up to the summer snow, none on it
        let p = ground(g, s.x, s.z);
        let bare = world::above_tree_line(s.x, s.z, p.y);
        if (s.v * 13.37).fract() < bare {
            continue;
        }
        let wooded = land.wooded(s.place);
        // 0 far from any wood, 1 at a wood's edge
        let by_wood = (wooded * 2.0).min(1.0);
        // a second and a third number of the slot's own
        let (u2, u3) = ((s.v * 7.13).fract(), (s.u * 11.7).fract());
        let clump = fields_smooth(0.6, 0.78, 0.5 + 0.5 * world::vnoise(s.x * 5.1, s.z * 5.1, 451));
        let stand = 0.5 + 0.5 * world::vnoise(s.x * 2.9 + 3.1, s.z * 2.9 + 8.7, 457);
        let (grow, scale) = match cls {
            _ if margin => {
                // scrub just outside a wood or a heath: bushes, and a low tree here and there
                if u2 < 0.3 && !near_plot(s.x, s.z) {
                    bush(&mut trees[0], &Site::level(p, 0.0), s.v, s.yaw, lk);
                }
                (s.u < 0.1, 0.5 + 0.25 * s.v)
            }
            world::WOOD => {
                let deep = (wooded * 2.0 - 1.0).clamp(0.0, 1.0);
                let open = world::glade(s.x, s.z, deep);
                ((1.0 - s.u) < (0.7 + 0.26 * deep) * (1.0 - open), (0.62 + 0.3 * deep) * (0.78 + 0.44 * stand) + 0.3 * s.v)
            }
            // the odd tree out in a field, and clumps and lone trees near a wood;
            // most of a field's trees stand in its hedges
            world::FARM | world::PASTURE => (s.u < 0.012 + by_wood * (0.03 + 0.5 * clump), 0.7 + 0.2 * stand + 0.25 * s.v),
            world::HEATH => {
                // gorse and broom: low bushes, and the odd tree, more of them by a wood
                if u2 < 0.16 && !near_plot(s.x, s.z) {
                    bush(&mut trees[0], &Site::level(p, 0.0), s.v, s.yaw, lk);
                }
                (s.u < 0.025 + by_wood * (0.03 + 0.3 * clump), 0.55 + 0.15 * stand + 0.25 * s.v)
            }
            _ => (false, 1.0),
        };
        if !grow || near_plot(s.x, s.z) {
            continue;
        }
        let scale = scale * (1.0 - 0.45 * bare);
        // conifers in stands, on high ground and on the poor tops
        let high = fields_smooth(0.34, 0.8, p.y + 0.12 * world::vnoise(s.x * 0.9, s.z * 0.9, 453));
        let poor = fields_smooth(0.55, 0.95, g.land.moor[s.place.parcel as usize]);
        let share = (0.05 + 0.75 * high + 0.2 * poor).min(0.92);
        let conifer = 0.65 * (0.5 + 0.5 * world::vnoise(s.x * 2.3 + 6.1, s.z * 2.3 + 1.9, 455)) + 0.35 * u3 < 0.2 + 0.6 * share;
        let species = if conifer { Species::Conifer } else { Species::Broadleaf };
        tree(&mut trees[leaf_group(s.v)], &Site::level(p, 0.0), &Tree { species, size: scale, tint: s.u, turn: s.yaw }, lk);
    }

    // (task A) the towns are handed the year's land, which says what town ground is built up
    let census = towns::houses(r, g, slots, cen, k, lk, &parishes, &near_plot_or_water, &land, &mut towns::Into { houses: &mut houses, smoke: &mut smoke, trees: &mut trees, lod });
    if let Some(c) = census.filter(|_| std::env::var_os("ISOSTATE_TOWNS").is_some()) {
        eprintln!(
            "towns {}: {} cells of town ground, {:.0} residents, {} houses wanted and {} drawn ({:.1} people to a house): {} cottages, {} townhouses, {} in terraces, {} villas; {} of one storey, {} of two, {} of three or more ({} of those with no neighbour on one side); {} houses short on cells with open ground left and {} on cells built up whole",
            r.y0 + k as i32, c.cells, c.residents, c.wanted, c.drawn, c.residents / c.drawn.max(1) as f32, c.cottages, c.townhouses, c.terraced, c.villas, c.storeys[0], c.storeys[1], c.storeys[2], c.tall_alone, c.short, c.short_full
        );
    }

    // hedgerow trees, once the fields are enclosed, and never in a road
    let year = r.y0 + k as i32;
    let roads = RoadIndex::new(r, cen, k);
    for (s, &open) in slots.hedge.iter().zip(&slots.hedge_open) {
        if year < open {
            continue;
        }
        let (cls, _) = land.at(s.place);
        if !(cls == world::FARM || cls == world::PASTURE) || near_plot(s.x, s.z) || roads.nearest(s.x, s.z, 0.03).is_some() {
            continue;
        }
        // the hedges are fuller of trees near a wood than out on the open plain
        if (s.v * 5.77).fract() > 0.6 + 0.8 * land.wooded(s.place) {
            continue;
        }
        let p = ground(g, s.x, s.z);
        if (s.u * 13.37).fract() < world::above_tree_line(s.x, s.z, p.y) {
            continue;
        }
        hedge_tree(&mut trees[leaf_group(s.v)], &Site::level(p, 0.0), 0.55 + 0.3 * s.v, s.u * 7.3, s.yaw, lk);
    }

    // farmsteads among the fields, each with a lane to the nearest road
    for (s, &open) in slots.farms.iter().zip(&slots.farm_open) {
        let c = s.wcell as usize;
        // the farm is of the land it stands on
        let (main, _) = land.at(s.place);
        if year < open || !(main == world::FARM || main == world::PASTURE) || s.u > 0.32 || near_plot(s.x, s.z) {
            continue;
        }
        if roads.nearest(s.x, s.z, 0.075).is_some() {
            continue;
        }
        // nobody farms from a house above the tree line: the mountain's grass is grazed from below
        let p = ground(g, s.x, s.z);
        if world::above_tree_line(s.x, s.z, p.y) > 0.0 {
            continue;
        }
        // stone and slate in the hills
        let upland = r.terrain[c] >= 2;
        let site = Site::on(p, s.yaw, &height);
        farmstead(&mut houses, &mut smoke, &mut trees[leaf_group(s.v)], &site, &Farmstead { arable: main == world::FARM, upland, detail: lod.at(s.x, s.z), seed: Seed::new(s.u, s.v) }, lk);
        let q = bevy::math::Quat::from_rotation_y(s.yaw);
        let gate = Vec3::new(s.x, 0.0, s.z) + q * Vec3::new(0.0, 0.0, -0.05);
        if let Some(to) = roads.nearest(gate.x, gate.z, 0.55) {
            lane(&mut lanes, lk, g, (gate.x, gate.z), to, s.v);
        }
    }

    let pit_kind = |cls: u8| match cls {
        world::COALMINE => Some(PitKind::Coal),
        world::IRONMINE => Some(PitKind::Ironstone),
        world::QUARRY => Some(PitKind::Stone),
        world::CLAYPIT => Some(PitKind::Clay),
        _ => None,
    };
    // Where the run keeps its quarries, pits and mines as workings (a pit head on
    // one cell, with its hands; the core's docs/rules/mines.md), each working at
    // work gets one pit, on the first free spot of its cell: a stand-in until the
    // pit heads and spoil heaps are drawn to their size. Otherwise the pits are
    // read off the land mix, a pit on every spot the mix paints as diggings.
    let year_workings = r.workings.get(k).filter(|w| !w.is_empty());
    if let Some(list) = year_workings {
        let mut waiting: std::collections::HashMap<u32, Vec<PitKind>> = std::collections::HashMap::new();
        for w in list.iter().filter(|w| w.opened != 0 && w.hands >= 1.0) {
            if let Some(kind) = pit_kind(w.class) {
                waiting.entry(w.cell).or_default().push(kind);
            }
        }
        for s in &slots.mines {
            if near_plot(s.x, s.z) {
                continue;
            }
            let Some(kinds) = waiting.get_mut(&s.wcell) else { continue };
            let Some(kind) = kinds.pop() else { continue };
            pit(&mut works_b, &Site::level(ground(g, s.x, s.z), s.yaw), &Pit { kind, v: s.v }, lk);
        }
    } else {
        for s in &slots.mines {
            let c = s.wcell as usize;
            let cls = world::pick_class(r, k, c, s.sub);
            let Some(kind) = pit_kind(cls) else { continue };
            if s.u > 0.7 || near_plot(s.x, s.z) {
                continue;
            }
            pit(&mut works_b, &Site::level(ground(g, s.x, s.z), s.yaw), &Pit { kind, v: s.v }, lk);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn houses_are_drawn_in_full_near_a_close_camera_and_as_shells_far_off() {
        let close = Lod::of_camera((10.0, 10.0), 1.5);
        assert_eq!(close.at(10.0, 10.0), Detail::Near);
        assert_eq!(close.at(12.5, 10.0), Detail::Near);
        assert_eq!(close.at(16.0, 10.0), Detail::Play);
        assert_eq!(close.at(40.0, 10.0), Detail::Far);
        let play = Lod::of_camera((10.0, 10.0), 14.0);
        assert_eq!(play.at(10.0, 10.0), Detail::Play);
        assert_eq!(play.at(10.0, 27.0), Detail::Play);
        assert_eq!(play.at(10.0, 35.0), Detail::Far);
        let far = Lod::of_camera((10.0, 10.0), 70.0);
        assert_eq!(far.at(10.0, 10.0), Detail::Far);
        assert_eq!(Lod::all(Detail::Near).at(99.0, 0.0), Detail::Near);
    }

    #[test]
    fn scenery_is_built_again_when_the_camera_sees_past_what_was_built_for_it() {
        let built = Lod::of_camera((10.0, 10.0), 1.5);
        // looking about a little, or drawing back a little, what was built still covers the view
        assert!(!built.stale(&Lod::of_camera((10.5, 10.3), 1.5)));
        assert!(!built.stale(&Lod::of_camera((10.0, 10.0), 2.2)));
        assert!(!built.stale(&built));
        // panned a few cells off, or drawn back to see more than was built in full
        assert!(built.stale(&Lod::of_camera((13.0, 10.0), 1.5)));
        assert!(built.stale(&Lod::of_camera((10.0, 10.0), 4.5)));
        // drawn right back, the island is built again as shells
        assert!(built.stale(&Lod::of_camera((10.0, 10.0), 70.0)));
        // and coming in again from far off, in full
        let far = Lod::of_camera((10.0, 10.0), 70.0);
        assert!(!far.stale(&Lod::of_camera((30.0, 10.0), 120.0)));
        assert!(far.stale(&Lod::of_camera((10.0, 10.0), 14.0)));
        assert!(Lod::of_camera((10.0, 10.0), 14.0).stale(&built));
        // the play distance covers a good pan before it must be built again
        let play = Lod::of_camera((10.0, 10.0), 14.0);
        assert!(!play.stale(&Lod::of_camera((16.0, 12.0), 14.0)));
        assert!(play.stale(&Lod::of_camera((22.0, 10.0), 14.0)));
        // just past the distance at which houses are drawn in full, what was built in full is kept
        let nearish = Lod::of_camera((10.0, 10.0), 3.8);
        assert!(!nearish.stale(&Lod::of_camera((10.0, 10.0), 4.4)));
        assert!(nearish.stale(&Lod::of_camera((10.0, 10.0), 5.5)));
    }

    #[test]
    fn each_try_for_a_yard_asks_for_less_room_than_the_last() {
        let tries = [Room::Proper, Room::Tight, Room::Any];
        for pair in tries.windows(2) {
            assert!(pair[1].dry() < pair[0].dry() && pair[1].shore() < pair[0].shore());
        }
        // and the last still keeps a yard out of the sea and out of a river
        assert!(Room::Any.dry() > 0.0 && Room::Any.shore() > 0.0);
    }
}
