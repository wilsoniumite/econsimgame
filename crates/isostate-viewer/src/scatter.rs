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
use isostate_gen::works::site::{self, Edge, Plot};
use isostate_gen::works::{self, pit, Kind, Pit, PitKind, State};
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
    /// the ground of each works in its own frame (x along its front, z out to
    /// its gate): its outline, and what each side of it faces and since when
    pub firm_plot: Vec<Plot>,
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
// Every works has a plot of its own, as wide and as deep as its kind needs at
// its size with room for its stores and yards (`site::ground`), and the plots of a cell must not overlap one
// another, the market place, a churchyard (in any year the church stands
// there: `towns::churchyards`), the roads, the railway or the river. So the
// ground of a cell is handed out: each works, in the order they were founded,
// is given the free spot it likes best that holds the plot it will need at
// the largest size it reaches in the record (`site::ground`). What it likes:
// to stand by what it was founded for (the stream, if a wheel is to turn it;
// the railway, if the line was there and its stuff is heavy; otherwise the
// road), and by the works already there, nearest of all one that makes what
// it uses or uses what it makes. So the works of a cell gather into a
// district along its road, with lanes between them, and a chain of trades
// stands wall to wall. Its gate faces the road. A works that has closed
// keeps its ground while its buildings stand (`STANDS_CLOSED` years: cold,
// then derelict) and then gives it up to those founded after. Nothing moves from year to
// year, and a works founded later never moves one that stands. In a live game the future is not known: when a works
// outgrows the ground set aside for it, its cell is laid out again, and its
// younger neighbours may move.
//
// A cell with more works than it has room for: a works that finds no spot is
// tried at seven tenths of the size, and again, down to the smallest; it is then
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

/// How long the buildings of a works that has closed are left standing: cold,
/// with the gate shut, for the first years, then derelict, and then cleared.
/// (The record says the year a works' capacity went to nothing and no more:
/// how long a ruin stands is the viewer's rule, the same for every works.)
const STANDS_COLD: usize = 2;
const STANDS_CLOSED: usize = 6;

/// What a works is doing that has had no capacity for `years`, while it stands.
fn closed_state(years: usize) -> Option<State> {
    match years {
        0 => None,
        y if y <= STANDS_COLD => Some(State::Idle),
        y if y <= STANDS_CLOSED => Some(State::Derelict),
        _ => None,
    }
}

/// The years a works keeps its ground (the first year index it has capacity,
/// and one past the last year its buildings stand, which is `STANDS_CLOSED`
/// years after it last had any; or no end if it is at work in the last year
/// known), and the most capacity it reaches.
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
        Some(from) => (from, if to >= r.filled.min(r.ny) { usize::MAX } else { to + STANDS_CLOSED }, top),
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
    // the spots a yard may be centred on: a 14 x 14 grid on dry land (in a shuffled order,
    // which settles a choice between spots a works likes equally)
    const N: usize = 14;
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
    // how far each spot is from a road, from the railway and from water
    let off = |x: f32, z: f32, rail: bool| ways.iter().filter(|w| (w.3 > 0.0) == rail).map(|w| (w.0 - x).hypot(w.1 - z)).fold(f32::MAX, f32::min);
    let near: Vec<(f32, f32, f32)> = spots.iter().map(|s| (off(s.0, s.1, false), off(s.0, s.1, true), world::water_dist(g, s.0, s.1))).collect();
    let stream = near.iter().any(|n| n.2 < 0.16);
    // the year the railway came to the cell, if it has
    let railed = (0..r.ny.min(r.filled.max(1))).find(|&k| r.trans[k * r.n + cell] & 4 != 0);
    let process: Vec<site::Process> = firms.iter().map(|&f| site::Process::standard(&r.header.methods[r.header.firms[f].method].name).unwrap_or_else(|| site::Process::of(Kind::Other))).collect();
    let process_of = |f: usize| &process[firms.iter().position(|&o| o == f).unwrap()];
    // one works makes what another uses
    let chained = |a: usize, b: usize| {
        let (pa, pb) = (process_of(a), process_of(b));
        pb.inputs.iter().any(|(good, _)| good.name == pa.output.name) || pa.inputs.iter().any(|(good, _)| good.name == pb.output.name)
    };
    let mut placed: Vec<usize> = Vec::new();
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
        // what it was founded for (a STAND-IN: the record's reason gives the sums that chose the
        // cell, not what within the cell drew it): the stream, the railway or the road
        let p = process_of(f);
        let heavy = p.inputs.iter().any(|(good, _)| good.weight >= 1.5) || p.output.weight >= 1.2;
        let wheel = stream && r.header.firms[f].born < 1790 && matches!(kind, Kind::TextileMill | Kind::ForgeRails | Kind::CharcoalFurnace | Kind::Bloomery | Kind::Millwright);
        let by_rail = heavy && railed.map_or(false, |k| k <= from);
        // the spots in the order it likes them: by what it was founded for, and by the works already there
        let mut order: Vec<(f32, usize)> = (0..spots.len())
            .map(|i| {
                let (s, n) = (&spots[i], near[i]);
                let drawn = if wheel { n.2.max(0.05) } else if by_rail { n.1.min(1.0) } else { n.0.min(1.0) };
                let by = lots.iter().zip(&placed).filter(|(l, _)| from < l.to && l.from < to).map(|(l, &o)| (((l.x - s.0).abs() - l.hx).max(0.0).hypot(((l.z - s.1).abs() - l.hz).max(0.0))) * if chained(f, o) { 0.6 } else { 1.0 }).fold(f32::MAX, f32::min);
                // the fire trades and the mills keep out past the edge of the town: it is the
                // town's own trades that stand among its houses
                let in_town = if matches!(kind, Kind::HandLoom | Kind::Smithy | Kind::Millwright | Kind::Shops) { 0.0 } else { town.map_or(0.0, |(tx, tz)| (0.34 - (tx - s.0).hypot(tz - s.1)).max(0.0) * 2.5) };
                (drawn + in_town + if by < f32::MAX { 0.8 * by } else { 0.0 } + 0.01 * s.4, i)
            })
            .collect();
        order.sort_by(|a, b| a.0.total_cmp(&b.0));
        // its gate faces the nearest road (or, with none, where its own number says)
        let facing = |x: f32, z: f32| -> i32 {
            let to = ways.iter().filter(|w| w.3 == 0.0).map(|w| (w.0 - x, w.1 - z)).min_by(|a, b| a.0.hypot(a.1).total_cmp(&b.0.hypot(b.1)));
            match to {
                Some((dx, dz)) if dx.hypot(dz) < 0.6 => (0..4).max_by(|&a, &b| { let f = |q: i32| { let (s, c) = (q as f32 * std::f32::consts::FRAC_PI_2).sin_cos(); s * dx + c * dz }; f(a).total_cmp(&f(b)) }).unwrap_or(quarter),
                _ => quarter,
            }
        };
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
            let (w, d) = site::ground(kind, size);
            for pass in 0..=liked.len() {
                for &(_, i) in &order {
                    let (x, z, wc, sub, _) = spots[i];
                    if pass < liked.len() && world::pick_class(r, year, wc, sub) != liked[pass] {
                        continue;
                    }
                    let q0 = facing(x, z);
                    for q in [q0, q0 + 1] {
                        let (hx, hz) = if q % 2 == 0 { (w * 0.5, d * 0.5) } else { (d * 0.5, w * 0.5) };
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
            // (task B2: by smaller steps than halves, so that a works is drawn as near its size as its cell has room for)
            size = (size * 0.7).max(0.4);
        }
        if found.is_none() {
            let (w, d) = site::ground(kind, size);
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
        let (w, d) = site::ground(kind, size);
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
        slots.firm_plot[f] = plot_of(g, ways, (x, z, slots.firm_pos[f].2), (w, d), slots.firm_water[f], railed.map(|k| r.y0 + k as i32));
        lots.push(Lot { x, z, hx, hz, from, to });
        placed.push(f);
    }
    // the works that stand together: where one makes what its neighbour uses and their plots
    // lie side by side, each has the other over its wall from the year both stand
    for (i, &a) in placed.iter().enumerate() {
        for (j, &b) in placed.iter().enumerate() {
            let (la, lb) = (&lots[i], &lots[j]);
            if i == j || !(la.from < lb.to && lb.from < la.to) || !process_of(b).inputs.iter().any(|(good, _)| good.name == process_of(a).output.name) {
                continue;
            }
            let (dx, dz) = (lb.x - la.x, lb.z - la.z);
            let (gx, gz) = (dx.abs() - la.hx - lb.hx, dz.abs() - la.hz - lb.hz);
            // side by side: a lane's width apart one way, and overlapping the other
            if !((gx < 0.05 && gz < -0.02) || (gz < 0.05 && gx < -0.02)) {
                continue;
            }
            let since = r.y0 + la.from.max(lb.from) as i32;
            for (f, edge, to) in [(a, Edge::Feeds, (dx, dz)), (b, Edge::Fed, (-dx, -dz))] {
                // in the works' own frame
                let (sin, cos) = slots.firm_pos[f].2.sin_cos();
                let own = (to.0 * cos - to.1 * sin, to.0 * sin + to.1 * cos);
                let plot = &mut slots.firm_plot[f];
                let side = plot.side_toward(own);
                if side != 0 && plot.edges[side] == Edge::Wall {
                    plot.edges[side] = edge;
                    plot.since[side] = since;
                }
            }
        }
    }
    if homeless > 0 && std::env::var_os("ISOSTATE_PLOTS").is_some() {
        let most = (0..r.ny).map(|k| lots.iter().filter(|l| l.from <= k && k < l.to).count()).max().unwrap_or(0);
        eprintln!("cell {cell}: {} works over the years, {most} at once at most, {} spots, town {}, {} points of road and rail, {homeless} with no ground", firms.len(), spots.len(), town.is_some(), ways.len());
    }
    (squeezed, homeless)
}

/// The plot of a works whose yard's middle, facing and size are settled: a
/// rectangle with one of its far corners cut off where a road, the railway
/// or water presses on it, and what each side faces. `rail` is the year the
/// railway came to the cell.
fn plot_of(g: &Ground, ways: &[Way], (x, z, yaw): (f32, f32, f32), (w, d): (f32, f32), water: bool, rail: Option<i32>) -> Plot {
    let (hw, hd) = (w * 0.5, d * 0.5);
    let (sin, cos) = yaw.sin_cos();
    // a point of the works' own frame on the map, and a map direction in the works' frame
    let map = |px: f32, pz: f32| (x + px * cos + pz * sin, z - px * sin + pz * cos);
    let own = |dx: f32, dz: f32| (dx * cos - dz * sin, dx * sin + dz * cos);
    // how hard the land presses on a far corner: how near a road, the railway or water comes to it
    let press = |px: f32, pz: f32| {
        let (mx, mz) = map(px, pz);
        let way = ways.iter().map(|w| (w.0 - mx).hypot(w.1 - mz) - w.2).fold(f32::MAX, f32::min);
        way.min(world::water_dist(g, mx, mz) - 0.045)
    };
    let (left, right) = (press(-hw, -hd), press(hw, -hd));
    let cut = |p: f32| (0.42 - p * 5.0).clamp(0.0, 0.4);
    let mut plot = if left <= right && cut(left) >= 0.12 {
        let k = cut(left);
        Plot::new(vec![(-hw, hd), (hw, hd), (hw, -hd), (-hw + w * k, -hd), (-hw, -hd + d * k)])
    } else if cut(right) >= 0.12 {
        let k = cut(right);
        Plot::new(vec![(-hw, hd), (hw, hd), (hw, -hd + d * k), (hw - w * k, -hd), (-hw, -hd)])
    } else {
        Plot::rect(w, d)
    };
    // the side nearest the railway, where the line runs within reach of the plot
    if let Some(year) = rail {
        let to = ways.iter().filter(|w| w.3 > 0.0).map(|w| (w.0 - x, w.1 - z)).min_by(|a, b| a.0.hypot(a.1).total_cmp(&b.0.hypot(b.1)));
        if let Some((dx, dz)) = to.filter(|t| t.0.hypot(t.1) < hw.hypot(hd) + 0.3) {
            let side = plot.side_toward(own(dx, dz));
            if side != 0 {
                plot = plot.facing_from(side, Edge::Rail, year);
            }
        }
    }
    // the side nearest the water: a stream, or a river broad enough to carry boats
    if water {
        let step = hw.hypot(hd) + 0.03;
        let to = (0..8).map(|i| { let a = i as f32 * std::f32::consts::FRAC_PI_4; (a.cos(), a.sin()) }).min_by(|a, b| world::water_dist(g, x + a.0 * step, z + a.1 * step).total_cmp(&world::water_dist(g, x + b.0 * step, z + b.1 * step)));
        if let Some((dx, dz)) = to {
            let side = plot.side_toward(own(dx, dz));
            let broad = g.rivers.iter().flat_map(|rv| rv.pts.iter().zip(&rv.half)).map(|(p, half)| ((p.x - x).hypot(p.y - z), *half)).min_by(|a, b| a.0.total_cmp(&b.0)).map_or(false, |(_, half)| half >= 0.03);
            // (a quay stands on the water: a plot whose side lies a field back from the bank has none)
            let (a, b) = (plot.outline[side], plot.outline[(side + 1) % plot.outline.len()]);
            let (mx, mz) = map((a.0 + b.0) * 0.5, (a.1 + b.1) * 0.5);
            let on_the_bank = world::water_dist(g, mx, mz) < 0.06;
            if side != 0 && plot.edges[side] == Edge::Wall {
                plot = plot.facing(side, Edge::Water { navigable: broad && on_the_bank });
            }
        }
    }
    plot
}

/// A works as the planner takes it in year `k`: its plan, and how it stands that year.
pub struct FirmSite {
    pub plan: std::sync::Arc<site::Plan>,
    pub now: site::Now,
}

/// The plans already made, by firm and the mark of what each was made from (its plot, the
/// stages of its growth so far, what its plot's sides face that year): a plan depends on a
/// works' whole history, so it is made once and kept, and a year looked at again finds it.
static PLANS: std::sync::Mutex<Option<std::collections::HashMap<(usize, u64), std::sync::Arc<site::Plan>>>> = std::sync::Mutex::new(None);

/// The works of firm `f` as the planner takes it in year `k`, if it stands
/// then, at work or closed and not yet cleared (`works::site`). THE FACTS,
/// and where each comes from:
/// - its kind and process: the method (`site::Process::standard`, the shipped content);
/// - the stages of its growth: its capacity in every year so far, on its size ladder;
/// - its plot: `Slots::firm_plot`, with what each side faces and since when;
/// - its state: going up in its first year; a stage built this year is under
///   scaffolding; stages it has shrunk out of stand idle, then derelict; and
///   the whole of it cold for `STANDS_COLD` years after its capacity went to
///   nothing, then derelict, until it is cleared `STANDS_CLOSED` years after;
/// - how busy it is and what it holds: STAND-INS, as the record keeps neither,
///   both read from what its capacity did this year. Busy: 0.9 in a year its
///   capacity rose, 0.45 in one it fell, else 0.75. Stock: a works that is
///   growing buys ahead and sells as fast as it makes (its stock yards four
///   fifths full, its warehouse a third); one that is shrinking runs its
///   stock down and has what it made on its hands (a third and five sixths);
///   otherwise three fifths and a half. Each works keeps a little more or
///   less than that, by a number of its own that never changes, so nothing
///   in a yard changes in a year the record does not.
pub fn firm_site(r: &Replay, slots: &Slots, f: usize, k: usize, lk: &Look) -> Option<FirmSite> {
    Some(planned(r, slots, f, k, lk, &firm_facts(r, slots, f, k)?))
}

/// The plan of a works whose facts are in hand (found among those already made, or made and kept), and how it stands in year `k`.
fn planned(r: &Replay, slots: &Slots, f: usize, k: usize, lk: &Look, facts: &Facts) -> FirmSite {
    let firm = &r.header.firms[f];
    let year = r.y0 + k as i32;
    let kept = PLANS.lock().unwrap().get_or_insert_with(Default::default).get(&(f, facts.mark)).cloned();
    let plan = kept.unwrap_or_else(|| {
        let kind = firm_kind(r, f);
        let process = site::Process::standard(&r.header.methods[firm.method].name).unwrap_or_else(|| site::Process::of(Kind::Other));
        let plan = std::sync::Arc::new(site::plan(kind, &process, &facts.stages, &slots.firm_plot[f], year, firm_seed(f), lk));
        PLANS.lock().unwrap().get_or_insert_with(Default::default).insert((f, facts.mark), plan.clone());
        plan
    });
    let history = &facts.history;
    let size = history.last().map_or(0.4, |h| h.1);
    let top = history.iter().map(|h| h.1).fold(0.0f32, f32::max);
    let shrunk_for = history.iter().rev().take_while(|h| h.1 < top * 0.8).count() as i32;
    if let Some(state) = closed_state(facts.closed_for) {
        // closed: cold, with what it could not sell about it, and then derelict
        return FirmSite { plan, now: site::Now { year, size, state, busy: 0.0, stock_in: 0.1, stock_out: 0.3, shrunk_for, detail: Detail::Near } };
    }
    let kk = replay::firm_k(r, f, k);
    let before = if k > 0 { replay::firm_k(r, f, k - 1) } else { kk };
    let state = if history.len() == 1 && firm.born > r.y0 { State::GoingUp } else { State::Working };
    let (rose, fell) = (kk > before * 1.02 && history.len() > 1, kk < before * 0.98);
    let busy = if rose { 0.9 } else if fell { 0.45 } else { 0.75 };
    // (a works keeps a little more or less about it than its neighbour, by a number of its own that never changes)
    let own = |n: i32| 0.12 * (world::hash2(f as i32, n, 53) - 0.5);
    let stock_in = (if rose { 0.8 } else if fell { 0.35 } else { 0.6 }) + own(1);
    let stock_out = (if rose { 0.3 } else if fell { 0.85 } else { 0.5 }) + own(2);
    FirmSite { plan, now: site::Now { year, size, state, busy, stock_in, stock_out, shrunk_for, detail: Detail::Near } }
}

/// What the plan of a works is made from in a year: its size in every year it
/// has stood so far, the stages of its growth, and a mark of those and of its
/// plot as its sides face that year, by which a plan already made is found.
struct Facts {
    history: Vec<(i32, f32)>,
    stages: Vec<site::Stage>,
    mark: u64,
    /// how many years it has had no capacity: nought while it is at work
    closed_for: usize,
}

/// The facts of a works that is at work in year `k`, or that was within the
/// last `STANDS_CLOSED` years and still stands.
fn firm_facts(r: &Replay, slots: &Slots, f: usize, k: usize) -> Option<Facts> {
    // the last year it had any capacity
    let last = (k.saturating_sub(STANDS_CLOSED)..=k).rev().find(|&i| replay::firm_k(r, f, i) > 0.0)?;
    let closed_for = k - last;
    let (phase, cap) = (firm_phase(f), slots.firm_cap.get(f).copied().unwrap_or(f32::MAX));
    let size_of = |kk: f32| works::rung_size(works::rung(firm_ratio(r, f, kk), phase), phase).min(cap);
    let first = r.firm_first.get(f).copied().unwrap_or(0).max(0) as usize;
    let history: Vec<(i32, f32)> = (first..=last).filter_map(|i| { let kk = replay::firm_k(r, f, i); (kk > 0.0).then(|| (r.y0 + i as i32, size_of(kk))) }).collect();
    let stages = site::stages(&history);
    let year = r.y0 + k as i32;
    let plot = slots.firm_plot.get(f)?;
    let mark = {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for s in &stages {
            (s.year, s.size.to_bits()).hash(&mut h);
        }
        for p in &plot.outline {
            (p.0.to_bits(), p.1.to_bits()).hash(&mut h);
        }
        for i in 0..plot.edges.len() {
            plot.edge(i, year).hash(&mut h);
        }
        (r.header.firms[f].method, r.header.meta.seed).hash(&mut h);
        h.finish()
    };
    Some(Facts { history, stages, mark, closed_for })
}

/// What the plans of year `k` hold, in a line, for ISOSTATE_TIMING: how many
/// works have a siding, a wharf, a neighbour over the wall, parts out of use,
/// and how many were built smaller than they are for want of room on their
/// plot. With ISOSTATE_PLOTS, where the first few of each stand, for pointing
/// a camera at them.
fn plans_said(r: &Replay, slots: &Slots, k: usize, lk: &Look) -> String {
    let year = r.y0 + k as i32;
    let names = ["a siding", "a wharf", "a works that feeds it over the wall", "one it feeds", "parts out of use", "a stage built smaller for want of room", "a stage that found no room at all", "its gate shut", "its roofs falling in"];
    let mut count = [0usize; 9];
    let mut places: [Vec<String>; 9] = Default::default();
    let mut all = 0;
    // the capacity the works have, what their stages come to, and what of it is built
    let (mut have, mut staged, mut built) = (0.0f32, 0.0f32, 0.0f32);
    for f in 0..r.header.firms.len() {
        let Some(fs) = firm_site(r, slots, f, k, lk) else { continue };
        all += 1;
        let plan = &fs.plan;
        if !matches!(fs.now.state, State::Idle | State::Derelict) {
            have += fs.now.size;
            staged += plan.stages.last().map_or(0.0, |s| s.size);
            built += plan.parts.iter().filter(|p| p.role == site::Role::Core).map(|p| p.size).sum::<f32>();
        }
        let faces = |want: Edge| (0..plan.plot.edges.len()).any(|i| plan.plot.edge(i, year) == want);
        let top = plan.stages.last().map_or(0.0, |s| s.size);
        let has = [plan.siding.is_some(), plan.wharf.is_some(), faces(Edge::Fed), faces(Edge::Feeds), fs.now.shrunk_for > 0 && plan.stages.len() > 1 && fs.now.size < top, plan.squeezed > 0, plan.dropped > 0, fs.now.state == State::Idle, fs.now.state == State::Derelict];
        for (i, _) in has.iter().enumerate().filter(|(_, h)| **h) {
            count[i] += 1;
            if places[i].len() < 6 {
                let (x, z, _) = slots.firm_pos[f];
                places[i].push(format!("{} at {x:.2},{z:.2}", firm_kind(r, f).label()));
            }
        }
    }
    let mut out = format!("works standing in {year}: {all}; {}; of the capacity of those at work {:.0}% is in the stages they were built in and {:.0}% stands as buildings", names.iter().zip(count).map(|(n, c)| format!("{c} with {n}")).collect::<Vec<_>>().join(", "), 100.0 * staged / have.max(1e-6), 100.0 * built / have.max(1e-6));
    if std::env::var_os("ISOSTATE_PLOTS").is_some() {
        for (n, p) in names.iter().zip(&places).filter(|(_, p)| !p.is_empty()) {
            out.push_str(&format!("\n  with {n}: {}", p.join("; ")));
        }
    }
    out
}

/// The works of firm `f` as it is drawn in year `k`, if it stands then: the
/// size from its ladder, the state from what the year did to it.
pub fn firm_works(r: &Replay, slots: &Slots, f: usize, k: usize) -> Option<works::Works> {
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
    Some(works::Works {
        kind: firm_kind(r, f),
        size: works::rung_size(rung, phase).min(slots.firm_cap.get(f).copied().unwrap_or(f32::MAX)),
        founded: firm.born,
        state,
        water: slots.firm_water.get(f).copied().unwrap_or(false),
        rail: r.trans[k * r.n + firm.cell] & 4 != 0,
        // the level is the camera's to choose: `build_year` sets it for each works
        detail: Detail::Near,
        seed: firm_seed(f),
    })
}

/// The flag points already found, by firm, each with the size, the state and
/// the place it was found for. Finding one builds the whole works, so a year
/// asks only for those that changed.
#[derive(Default)]
pub struct Flags(std::collections::HashMap<usize, ((usize, u32, State, i32, [u32; 3]), Vec3)>);

/// Where the family's flag flies on the works of firm `f` in year `k`: the
/// top of its highest gable.
pub fn firm_flag(r: &Replay, g: &Ground, slots: &Slots, f: usize, k: usize, lk: &Look, known: &mut Flags) -> Option<Vec3> {
    let fs = firm_site(r, slots, f, k, lk)?;
    let &(x, z, yaw) = slots.firm_pos.get(f)?;
    // (what its height hangs on: its parts, its size, what it is doing and how long it has stood shrunk)
    let newest = fs.plan.stages.last().map_or(0, |s| s.year);
    let key = (fs.plan.parts.len() + if newest == fs.now.year { 1000 } else { 0 }, fs.now.size.to_bits(), fs.now.state, fs.now.shrunk_for.min(8), [x.to_bits(), z.to_bits(), yaw.to_bits()]);
    if let Some(&(_, at)) = known.0.get(&f).filter(|(was, _)| *was == key) {
        return Some(at);
    }
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    let at = site::flag_point(&Site::on(ground(g, x, z), yaw, &height), &fs.plan, &fs.now, lk);
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
    slots.firm_plot.resize(n, Plot::rect(0.1, 0.08));
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
                let (w, d) = site::ground(firm_kind(r, f), works::rung_size(rungs[f], firm_phase(f)).min(slots.firm_cap[f]));
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
    if std::env::var_os("ISOSTATE_PLOTS").is_some() {
        // the cells with the most works at the end, and what stands in each, for pointing a camera
        let last = r.filled.clamp(1, r.ny) - 1;
        let mut cells: std::collections::BTreeMap<usize, Vec<usize>> = Default::default();
        for f in (0..n).filter(|&f| replay::firm_k(r, f, last) > 0.0) {
            cells.entry(r.header.firms[f].cell).or_default().push(f);
        }
        let mut busiest: Vec<(usize, Vec<usize>)> = cells.into_iter().collect();
        busiest.sort_by_key(|c| std::cmp::Reverse(c.1.len()));
        for (cell, firms) in busiest.iter().take(24) {
            let mut kinds: std::collections::BTreeMap<&str, (usize, f32, i32)> = Default::default();
            for &f in firms {
                let e = kinds.entry(firm_kind(r, f).label()).or_insert((0, 0.0, i32::MAX));
                *e = (e.0 + 1, e.1.max(firm_ratio(r, f, replay::firm_k(r, f, last))), e.2.min(r.header.firms[f].born));
            }
            let (x, z, _) = slots.firm_pos[firms[0]];
            eprintln!("works in cell {},{} (one at {:.2},{:.2}): {}", cell % r.w, cell / r.w, x, z, kinds.iter().map(|(k, v)| format!("{} {k} (to size {:.1}, from {})", v.0, v.1, v.2)).collect::<Vec<_>>().join("; "));
        }
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
        firm_plot: Vec::new(),
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
    // ISOSTATE_WORKS=old draws every works the way it was drawn before the planner (one fixed
    // arrangement for its kind in the middle of its plot), to set the two side by side and to
    // time one against the other on the same machine in the same minute
    let old = std::env::var("ISOSTATE_WORKS").map_or(false, |v| v == "old");
    let begun = std::time::Instant::now();
    // what each works' plan is made from this year, and the plans themselves. Where many are
    // not made yet (a game just opened, a replay opened on a late year or moved a long way)
    // they are made on every core; most years want a handful
    let sites: Vec<Option<FirmSite>> = if old {
        Vec::new()
    } else {
        use rayon::prelude::*;
        let facts: Vec<Option<Facts>> = (0..r.header.firms.len()).into_par_iter().map(|f| firm_facts(r, slots, f, k)).collect();
        let wanting = {
            let plans = PLANS.lock().unwrap();
            facts.iter().enumerate().filter(|(f, facts)| facts.as_ref().map_or(false, |facts| plans.as_ref().map_or(true, |p| !p.contains_key(&(*f, facts.mark))))).count()
        };
        let site_of = |(f, facts): (usize, &Option<Facts>)| facts.as_ref().map(|facts| planned(r, slots, f, k, lk, facts));
        if wanting >= 24 { facts.par_iter().enumerate().map(site_of).collect() } else { facts.iter().enumerate().map(site_of).collect() }
    };
    for (f, firm) in r.header.firms.iter().enumerate() {
        if old {
            let Some(w) = firm_works(r, slots, f, k) else { continue };
            let p = slots.firm_pos[f];
            let (fw, fd) = works::footprint(w.kind, w.size);
            let (sin, cos) = p.2.sin_cos();
            let (hx, hz) = ((fw * cos.abs() + fd * sin.abs()) * 0.5 + 0.02, (fw * sin.abs() + fd * cos.abs()) * 0.5 + 0.02);
            plots.entry(firm.cell).or_default().push((p.0, p.1, hx, hz));
            let w = works::Works { detail: lod.at(p.0, p.1), ..w };
            works::works(&mut works_b, &mut smoke, &mut lanes, &Site::on(ground(g, p.0, p.1), p.2, &height), &w, lk);
            continue;
        }
        let Some(fs) = &sites[f] else { continue };
        let p = slots.firm_pos[f];
        // the box round the ground it has taken in so far as it lies on the map, and a little
        // room (the rest of its plot is kept for it, and is field or wood until it is wanted)
        let (x0, x1, z0, z1) = fs.plan.enclosed.bounds();
        let (fw, fd, mx, mz) = (x1 - x0, z1 - z0, (x0 + x1) * 0.5, (z0 + z1) * 0.5);
        let (sin, cos) = p.2.sin_cos();
        let (hx, hz) = ((fw * cos.abs() + fd * sin.abs()) * 0.5 + 0.02, (fw * sin.abs() + fd * cos.abs()) * 0.5 + 0.02);
        plots.entry(firm.cell).or_default().push((p.0 + mx * cos + mz * sin, p.1 - mx * sin + mz * cos, hx, hz));
        let site_ = Site::on(ground(g, p.0, p.1), p.2, &height);
        // as much of it as the camera can see, as for a house
        let now = site::Now { detail: lod.at(p.0, p.1), ..fs.now };
        site::site(&mut works_b, &mut smoke, &mut lanes, &site_, &fs.plan, &now, lk);
    }
    if std::env::var_os("ISOSTATE_TIMING").is_some() || std::env::var_os("ISOSTATE_BENCH").is_some() {
        eprintln!("timing year works: {:.1} ms for {} vertices of works and {} of their ground", begun.elapsed().as_secs_f64() * 1000.0, works_b.pos.len(), lanes.pos.len());
        if !old {
            eprintln!("{}", plans_said(r, slots, k, lk));
        }
    }
    // the railway's banks, viaducts, portals and bridges and what stands by it, and a
    // bridge wherever a road crosses a river
    let rail_year = crate::rails::year(r, g, cen, k);
    works_b.append(crate::rails::structures(&rail_year, g, lk));
    crate::rails::standing(&rail_year, lod, lk, &mut works_b, &mut smoke);
    for (c, m, lv) in crate::network::road_segments(r, k) {
        let (a, b) = (bevy::math::Vec2::new(cen[c].0, cen[c].1), bevy::math::Vec2::new(cen[m].0, cen[m].1));
        let along = (b - a).normalize_or(bevy::math::Vec2::X);
        for (at, river, half, level) in crate::rivers::crossings(&g.rivers, &g.water, a, b) {
            // a road that crosses aslant has further to go from bank to bank (up to three times as far)
            let half = half / along.perp_dot(river).abs().max(0.34);
            road_bridge(&mut works_b, &RoadBridge { at, along, half, level, main: lv == 3, detail: lod.at(at.x, at.y) }, &height, lk);
        }
        causeway(&mut works_b, lk, g, a, b, lv == 3);
    }
    let parishes = towns::landmarks(r, g, cen, k, lk, lod, &mut houses, &mut plots);
    // near a works or a market place, in a churchyard, or on the railway or its banks
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

    towns::houses(r, g, slots, cen, k, lk, &parishes, &near_plot_or_water, &mut towns::Into { houses: &mut houses, smoke: &mut smoke, trees: &mut trees, lod });

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
            pit(&mut works_b, &Site::level(ground(g, s.x, s.z), s.yaw), &Pit { kind, v: s.v, detail: lod.at(s.x, s.z) }, lk);
        }
    } else {
        for s in &slots.mines {
            let c = s.wcell as usize;
            let cls = world::pick_class(r, k, c, s.sub);
            let Some(kind) = pit_kind(cls) else { continue };
            if s.u > 0.7 || near_plot(s.x, s.z) {
                continue;
            }
            pit(&mut works_b, &Site::level(ground(g, s.x, s.z), s.yaw), &Pit { kind, v: s.v, detail: lod.at(s.x, s.z) }, lk);
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
    fn a_works_that_has_closed_stands_cold_then_derelict_and_is_then_cleared() {
        assert_eq!(closed_state(0), None, "a works at work is not closed");
        assert_eq!(closed_state(1), Some(State::Idle));
        assert_eq!(closed_state(STANDS_COLD), Some(State::Idle));
        assert_eq!(closed_state(STANDS_COLD + 1), Some(State::Derelict));
        assert_eq!(closed_state(STANDS_CLOSED), Some(State::Derelict));
        assert_eq!(closed_state(STANDS_CLOSED + 1), None, "its ground is given up");
    }

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
