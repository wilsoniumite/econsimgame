//! What a click on the map picks, the second stage of the game leaving its
//! grid. A click picks a thing and never a square: in this order a works, by
//! the ground its yard is drawn on; a quarry, a clay pit or a mine at work, by
//! its pit; a town, by any of its ground (or within 700 m of its market place,
//! as before, or by its name on the map, `main::label_clicks`); and in open
//! country the country of its market town, which the simulation keeps as each
//! cell's market (`catch`). No farm, parish or single house is made a thing:
//! the simulation keeps none of them.
//!
//! A spot in open country (`Pick::Place`) keeps the point clicked and the
//! simulation's 2 km cell under it. The card is the country's ("The country
//! round Oakbury") and ends with lines for the spot, "Round here", which carry
//! everything the card of a square told; the tools of the hand and the family
//! game's land agent are given the cell, so the orders and edits sent are
//! those a click on the square sent.
//!
//! None of this is the simulation's: nothing here moves a figure of a run.

use std::collections::HashMap;

use bevy::prelude::*;
use isostate_gen::works::site::Plot;

use crate::replay::{self, Replay};
use crate::scatter::{self, Slots};
use crate::world::{self, Ground};
use crate::{fmt_pop, Pick, DISP_NAMES};

/// How near a pit's middle a click must fall to pick it, in map units (a unit
/// is 2 km): 140 m, about the pit as drawn with its heaps.
pub const PIT_REACH: f32 = 0.07;
/// How near its market place a click picks a town even off its ground: 700 m,
/// as every click did before.
pub const MARKET_REACH: f32 = 0.35;
/// How far outside a works' yard a click still picks it: 20 m.
const YARD_MARGIN: f32 = 0.01;

/// A pit drawn in a year: where it stands, and the working it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitSpot {
    pub x: f32,
    pub z: f32,
    pub cell: usize,
    pub class: u8,
}

/// What a click may find near it in a year: the pits drawn, and the works
/// standing, listed under every cell their plot reaches into, so that a click
/// looks at the few works of its own cell and not at every works of the run
/// (in 2049 the run has thousands, and looking at each cost a millisecond and
/// a half a frame).
#[derive(Default)]
pub struct Near {
    pub pits: Vec<PitSpot>,
    works: HashMap<usize, Vec<usize>>,
}

/// What a click may find near it in the year on the clock, worked out once a
/// year (or once a run, a live year) and not every frame.
#[derive(Resource, Default)]
pub struct Pits {
    key: Option<(usize, usize, usize)>,
    pub near: Near,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Pits>().add_systems(Update, keep_pits.before(crate::pick));
}

/// The pits again when the year on the clock, the run or the slots change.
fn keep_pits(data: Res<crate::Data>, tl: Res<crate::Timeline>, mut pits: ResMut<Pits>) {
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    let key = (k, std::sync::Arc::as_ptr(&data.r) as usize, std::sync::Arc::as_ptr(&data.slots) as usize);
    if pits.key != Some(key) {
        pits.near = near_of(&data.r, &data.slots, k);
        pits.key = Some(key);
    }
}

/// Where the pits of year `k` are drawn: each working at work gets one pit, on
/// the first spot of its cell that no works' yard covers, as the scenery
/// places them (`scatter::build_year`). The scenery also keeps a pit off a
/// churchyard and the railway's banks, which this does not ask, so now and
/// then a pit is found a spot or two from where it is drawn.
pub fn pits_of(r: &Replay, slots: &Slots, k: usize) -> Vec<PitSpot> {
    let Some(list) = r.workings.get(k).filter(|w| !w.is_empty()) else { return Vec::new() };
    let mut waiting: HashMap<u32, Vec<u8>> = HashMap::new();
    for w in list.iter().filter(|w| w.opened != 0 && w.hands >= 1.0 && (world::QUARRY..=world::COALMINE).contains(&w.class)) {
        waiting.entry(w.cell).or_default().push(w.class);
    }
    if waiting.is_empty() {
        return Vec::new();
    }
    // the works standing this year, by the cell each stands in, whose yards a pit keeps clear of
    let mut yards: HashMap<usize, Vec<usize>> = HashMap::new();
    for f in 0..r.header.firms.len().min(slots.firm_pos.len()) {
        if replay::firm_k(r, f, k) > 0.0 {
            yards.entry(r.header.firms[f].cell).or_default().push(f);
        }
    }
    let mut out = Vec::new();
    for s in &slots.mines {
        let Some(kinds) = waiting.get_mut(&s.wcell) else { continue };
        if kinds.is_empty() {
            continue;
        }
        if yards.get(&cell_at(r, s.x, s.z)).is_some_and(|fs| fs.iter().any(|&f| on_plot(slots, f, &slots.firm_plot[f], s.x, s.z, 0.02))) {
            continue;
        }
        if let Some(class) = kinds.pop() {
            out.push(PitSpot { x: s.x, z: s.z, cell: s.wcell as usize, class });
        }
    }
    out
}

/// What a click may find near it in year `k`: the pits, and the works
/// standing (not the shops and offices of "services", which no works stands
/// for) by the cells their plots reach into.
pub fn near_of(r: &Replay, slots: &Slots, k: usize) -> Near {
    let mut works: HashMap<usize, Vec<usize>> = HashMap::new();
    for (f, firm) in r.header.firms.iter().enumerate() {
        let (Some(&(px, pz, _)), Some(plot)) = (slots.firm_pos.get(f), slots.firm_plot.get(f)) else { continue };
        if replay::firm_k(r, f, k) <= 0.0 || r.header.methods.get(firm.method).is_none_or(|m| m.name == "services") {
            continue;
        }
        // no plot reaches further from its middle than its corners
        let (x0, x1, z0, z1) = plot.bounds();
        let reach = x0.abs().max(x1.abs()).hypot(z0.abs().max(z1.abs())) + YARD_MARGIN;
        let (cx0, cx1) = ((px - reach).floor().max(0.0) as usize, ((px + reach).floor().max(0.0) as usize).min(r.w - 1));
        let (cz0, cz1) = ((pz - reach).floor().max(0.0) as usize, ((pz + reach).floor().max(0.0) as usize).min(r.h - 1));
        for cz in cz0..=cz1 {
            for cx in cx0..=cx1 {
                works.entry(cz * r.w + cx).or_default().push(f);
            }
        }
    }
    Near { pits: pits_of(r, slots, k), works }
}

/// The cell a point lies in, kept on the map.
pub fn cell_at(r: &Replay, x: f32, z: f32) -> usize {
    (z.floor().max(0.0) as usize).min(r.h - 1) * r.w + (x.floor().max(0.0) as usize).min(r.w - 1)
}

/// A point on the map in works `f`'s own frame (x along its front, z out to its gate).
fn local(slots: &Slots, f: usize, x: f32, z: f32) -> (f32, f32) {
    let (px, pz, yaw) = slots.firm_pos[f];
    let (sin, cos) = yaw.sin_cos();
    let (dx, dz) = (x - px, z - pz);
    (dx * cos - dz * sin, dx * sin + dz * cos)
}

/// A point of works `f`'s own frame on the map (`Site::at`).
fn on_map(slots: &Slots, f: usize, (lx, lz): (f32, f32)) -> (f32, f32) {
    let (px, pz, yaw) = slots.firm_pos[f];
    let (sin, cos) = yaw.sin_cos();
    (px + lx * cos + lz * sin, pz - lx * sin + lz * cos)
}

/// Whether a point of the map lies on `plot` of works `f`, or within `margin` of it.
fn on_plot(slots: &Slots, f: usize, plot: &Plot, x: f32, z: f32, margin: f32) -> bool {
    let (lx, lz) = local(slots, f, x, z);
    let (x0, x1, z0, z1) = plot.bounds();
    if lx < x0 - margin || lx > x1 + margin || lz < z0 - margin || lz > z1 + margin {
        return false;
    }
    inside(&plot.outline, (lx, lz)) || near_edge(&plot.outline, (lx, lz), margin)
}

/// Whether a point lies inside a closed outline (by the crossings of a ray).
fn inside(poly: &[(f32, f32)], p: (f32, f32)) -> bool {
    let n = poly.len();
    let mut odd = false;
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < a.0 + (p.1 - a.1) / (b.1 - a.1) * (b.0 - a.0) {
            odd = !odd;
        }
    }
    odd
}

/// Whether a point lies within `d` of an outline's sides.
fn near_edge(poly: &[(f32, f32)], p: (f32, f32), d: f32) -> bool {
    let n = poly.len();
    (0..n).any(|i| {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        let (ex, ez) = (b.0 - a.0, b.1 - a.1);
        let t = (((p.0 - a.0) * ex + (p.1 - a.1) * ez) / (ex * ex + ez * ez).max(1e-12)).clamp(0.0, 1.0);
        (a.0 + ex * t - p.0).hypot(a.1 + ez * t - p.1) <= d
    })
}

/// The ground works `f` is drawn on in year `k`, in its own frame: the yard
/// it has taken in (`site::Plan::enclosed`) where the year's scenery has made
/// its plan, else the whole plot set aside for it.
fn yard(r: &Replay, slots: &Slots, f: usize, k: usize) -> Plot {
    scatter::firm_yard(r, slots, f, k).unwrap_or_else(|| slots.firm_plot[f].clone())
}

/// The outline of the ground works `f` is drawn on in year `k`, on the map.
pub fn yard_outline(r: &Replay, slots: &Slots, f: usize, k: usize) -> Vec<(f32, f32)> {
    if f >= slots.firm_pos.len() || f >= slots.firm_plot.len() {
        return Vec::new();
    }
    yard(r, slots, f, k).outline.iter().map(|&p| on_map(slots, f, p)).collect()
}

/// The works whose yard a point of the map lies on in year `k`, if any,
/// among those `near` lists for the point's cell.
fn works_at(r: &Replay, slots: &Slots, near: &Near, k: usize, x: f32, z: f32) -> Option<usize> {
    let mut best: Option<(f32, usize)> = None;
    for &f in near.works.get(&cell_at(r, x, z)).map_or(&[][..], |v| v.as_slice()) {
        let (Some(&(px, pz, _)), Some(plot)) = (slots.firm_pos.get(f), slots.firm_plot.get(f)) else { continue };
        let d = (px - x).hypot(pz - z);
        if best.is_some_and(|b| d >= b.0) {
            continue;
        }
        // on its whole plot, and then on the yard it is drawn on
        if on_plot(slots, f, plot, x, z, YARD_MARGIN) && on_plot(slots, f, &yard(r, slots, f, k), x, z, YARD_MARGIN) {
            best = Some((d, f));
        }
    }
    best.map(|b| b.1)
}

/// The market town of cell `c` in year `k`, if it has one standing.
pub fn market_of(r: &Replay, k: usize, c: usize) -> Option<usize> {
    let t = (*r.catch.get(k * r.n + c)? as usize).checked_sub(1)?;
    active(r, k, t).then_some(t)
}

fn active(r: &Replay, k: usize, t: usize) -> bool {
    t < r.header.towns.len() && t < r.n_towns && r.town_active.get(k * r.n_towns + t).is_some_and(|&a| a != 0)
}

/// The town whose ground a point of the map is in year `k`, if it is a
/// town's: the ground the record's land mix draws as town there (the parcel's
/// own class, `world::pick_class`), which is the town's whose market the cell
/// that decides it trades in.
pub fn town_ground_at(r: &Replay, g: &Ground, k: usize, x: f32, z: f32) -> Option<usize> {
    let (wc, sub) = world::class_key(g, x, z);
    if world::pick_class(r, k, wc, sub) != world::URBAN {
        return None;
    }
    market_of(r, k, wc)
}

/// What a click at (x, z) picks in year `k`: a works, a pit, a town, or the
/// spot in open country; nothing off the map or at sea.
#[allow(clippy::too_many_arguments)]
pub fn choose(r: &Replay, g: &Ground, slots: &Slots, cen: &[(f32, f32)], near: &Near, x: f32, z: f32, k: usize) -> Option<Pick> {
    if !(x >= 0.0 && z >= 0.0 && (x as usize) < r.w && (z as usize) < r.h) {
        return None;
    }
    if let Some(f) = works_at(r, slots, near, k, x, z) {
        // a university stands as a works but is its town's, whose card tells of it (and in a live
        // game its place in the list of works moves as the simulation's works come before it)
        if replay::is_university(r, f) {
            if let Some(t) = usize::try_from(r.header.firms[f].town).ok().filter(|&t| t < r.header.towns.len()) {
                return Some(Pick::Town(t));
            }
        }
        return Some(Pick::Firm(f));
    }
    let pit = near.pits.iter().map(|p| ((p.x - x).hypot(p.z - z), p)).filter(|(d, _)| *d < PIT_REACH).min_by(|a, b| a.0.total_cmp(&b.0));
    if let Some((_, p)) = pit {
        return Some(Pick::Pit { cell: p.cell, class: p.class });
    }
    if let Some(t) = town_ground_at(r, g, k, x, z) {
        return Some(Pick::Town(t));
    }
    for t in (0..r.n_towns.min(r.header.towns.len())).filter(|&t| active(r, k, t)) {
        let (mx, mz) = cen[r.header.towns[t].node];
        if (mx - x).hypot(mz - z) < MARKET_REACH {
            return Some(Pick::Town(t));
        }
    }
    let c = cell_at(r, x, z);
    (r.terrain[c] > 0).then_some(Pick::Place { x, z, cell: c })
}

/// A spot in open country at the middle of cell `c`: what an old script's or a
/// test's click on a square now is.
#[cfg_attr(not(test), allow(dead_code))]
pub fn at_cell(c: usize, w: usize) -> Pick {
    Pick::Place { x: (c % w) as f32 + 0.5, z: (c / w) as f32 + 0.5, cell: c }
}

/// A thing picked by its name, for pictures and scripts with nobody at the
/// window: "town:Oakbury" or "works:Fairfax Mill" (the first whose name
/// starts so, ignoring case), or "pit:coal" (the first pit at work of a kind:
/// stone, clay, iron or coal, or the N-th with "pit:coal:3"); or the error
/// that says what there is.
pub fn by_name(r: &Replay, pits: &[PitSpot], k: usize, what: &str) -> Result<Pick, String> {
    let starts = |name: &str, with: &str| name.to_lowercase().starts_with(&with.trim().to_lowercase());
    match what.split_once(':') {
        Some(("town", name)) => r.header.towns.iter().position(|t| starts(&t.name, name)).map(Pick::Town).ok_or_else(|| format!("no town is called {name}")),
        Some(("works", name)) => r.header.firms.iter().enumerate().find(|(f, firm)| starts(&firm.name, name) && replay::firm_k(r, *f, k) > 0.0).map(|(f, _)| Pick::Firm(f)).ok_or_else(|| format!("no works standing in {} is called {name}", r.y0 + k as i32)),
        Some(("pit", kind)) => {
            let (kind, nth) = kind.split_once(':').map_or((kind, 1), |(k, n)| (k, n.trim().parse::<usize>().unwrap_or(1).max(1)));
            pits.iter().filter(|p| pit_words(p.class).1.starts_with(&kind.trim().to_lowercase())).nth(nth - 1).map(|p| Pick::Pit { cell: p.cell, class: p.class }).ok_or_else(|| format!("no {nth}th pit of {kind} is at work in {}", r.y0 + k as i32))
        }
        _ => Err(format!("{what}: pick takes x,z, town:NAME, works:NAME or pit:KIND")),
    }
}

/// Where a picked thing stands on the map: a town's market place, a works'
/// yard, a pit, a spot.
pub fn spot_of(r: &Replay, slots: &Slots, cen: &[(f32, f32)], pits: &[PitSpot], p: Pick) -> Option<(f32, f32)> {
    match p {
        Pick::Town(t) => cen.get(r.header.towns.get(t)?.node).copied(),
        Pick::Firm(f) => slots.firm_pos.get(f).map(|q| (q.0, q.1)),
        Pick::Pit { cell, class } => pits.iter().find(|q| q.cell == cell && q.class == class).map(|q| (q.x, q.z)),
        Pick::Place { x, z, .. } => Some((x, z)),
    }
}

/// The words of a working's kind, and of what it raises.
fn pit_words(class: u8) -> (&'static str, &'static str) {
    match class {
        world::QUARRY => ("Quarry", "stone"),
        world::CLAYPIT => ("Clay pit", "clay"),
        world::IRONMINE => ("Iron mine", "iron ore"),
        _ => ("Colliery", "coal"),
    }
}

/// The compass point from one spot to another, in words: "north-east".
fn bearing(dx: f32, dz: f32) -> &'static str {
    // z runs south on the map; the angle is measured from north, clockwise
    let a = dx.atan2(-dz).to_degrees().rem_euclid(360.0);
    ["north", "north-east", "east", "south-east", "south", "south-west", "west", "north-west"][((a + 22.5) / 45.0) as usize % 8]
}

/// Where a spot lies, in words: from its market town's market place, and from
/// the island's north-west corner, in km.
fn whereabouts(r: &Replay, cen: &[(f32, f32)], town: Option<usize>, x: f32, z: f32) -> String {
    let corner = format!("{:.0} km east and {:.0} km south of the island's north-west corner", x * 2.0, z * 2.0);
    match town.and_then(|t| Some((r.header.towns.get(t)?, cen.get(r.header.towns[t].node)?))) {
        Some((t, &(mx, mz))) => {
            let d = (x - mx).hypot(z - mz) * 2.0;
            if d < 0.5 {
                format!("At {}'s market place, {corner}.", t.name)
            } else {
                format!("{} km {} of {}'s market place, {corner}.", if d < 10.0 { format!("{d:.1}") } else { format!("{d:.0}") }, bearing(x - mx, z - mz), t.name)
            }
        }
        None => format!("{}{}.", corner[..1].to_uppercase(), &corner[1..]),
    }
}

/// The land's uses at cell `c` in year `k`, as the record mixes them: "58%
/// woodland and 42% pasture in 2049".
fn uses(r: &Replay, k: usize, c: usize) -> String {
    let off = k * r.n + c;
    let (a, b, sh) = (r.mix_a[off] as usize, r.mix_b[off] as usize, r.mix_s[off] as f32 / 255.0);
    let year = r.y0 + k as i32;
    if a == b {
        format!("all {} in {year}", DISP_NAMES[a % 12])
    } else {
        format!("{:.0}% {} and {:.0}% {} in {year}", sh * 100.0, DISP_NAMES[a % 12], (1.0 - sh) * 100.0, DISP_NAMES[b % 12])
    }
}

/// What a click on open country shows: the country of its market town, and
/// for the spot, "Round here": its uses and where it lies (all that the card
/// of a square said, which was "Woodland near Oakbury", its two uses and its
/// place from the north-west corner). The first line is the card's title.
pub fn describe_place(r: &Replay, cen: &[(f32, f32)], x: f32, z: f32, c: usize, k: usize) -> String {
    let year = r.y0 + k as i32;
    let town = market_of(r, k, c);
    let mut s = match town {
        Some(t) => {
            let i = k * r.n_towns + t;
            let name = &r.header.towns[t].name;
            let (pop, urban) = (r.town_pop.get(i).copied().unwrap_or(0.0), r.town_urban.get(i).copied().unwrap_or(0.0));
            format!("The country round {name}\nIt buys and sells at {name}'s market: {} people live in its market area, {} of them in the town, in {year}.", fmt_pop(pop), fmt_pop(urban))
        }
        None => "Open country\nNo market reaches it.".to_string(),
    };
    s.push_str(&format!("\n\nRound here: {}.\n{}", uses(r, k, c), whereabouts(r, cen, town, x, z)));
    s
}

/// What a click on a pit shows: the working, its hands and what it raised,
/// and where it lies.
pub fn describe_pit(r: &Replay, cen: &[(f32, f32)], cell: usize, class: u8, k: usize) -> String {
    let year = r.y0 + k as i32;
    let (kind, good) = pit_words(class);
    let town = market_of(r, k, cell);
    let mut s = match town {
        Some(t) => format!("{kind} near {}\n", r.header.towns[t].name),
        None => format!("{kind}\n"),
    };
    let working = r.workings.get(k).and_then(|list| list.iter().find(|w| w.cell as usize == cell && w.class == class && w.opened != 0));
    match working {
        Some(w) => {
            s.push_str(&format!("Worked since {}, with {} hands in {year}.", w.opened, fmt_pop(w.hands)));
            if w.out > 0.0 {
                s.push_str(&format!("\nIt raised {} units of {good} that year, {} since it was opened.", fmt_pop(w.out), fmt_pop(w.heap)));
            }
        }
        None => s.push_str(&format!("No longer worked in {year}.")),
    }
    let (x, z) = cen.get(cell).copied().unwrap_or(((cell % r.w) as f32 + 0.5, (cell / r.w) as f32 + 0.5));
    s.push_str(&format!("\n{}", whereabouts(r, cen, town, x, z)));
    s
}

/// The outline of town `t`'s ground in year `k`, as closed lines on the map:
/// the ground is looked at every 125 m over its cells and their neighbours,
/// its edge traced between the looks (marching squares) and rounded twice
/// (Chaikin's corner cutting), so that it follows the parcels the ground is
/// drawn in and not the cells. A city of 88 cells is some 40,000 looks, tens
/// of milliseconds: it is worked out off the main thread (cursor.rs).
pub fn town_outline(r: &Replay, g: &Ground, k: usize, t: usize) -> Vec<Vec<(f32, f32)>> {
    const PER: usize = 16;
    let step = 1.0 / PER as f32;
    // the cells of town ground that trade at the town, and its market place's
    let (mut x0, mut z0, mut x1, mut z1) = (usize::MAX, usize::MAX, 0usize, 0usize);
    let mut any = false;
    for c in 0..r.n {
        let off = k * r.n + c;
        let urban = r.mix_a.get(off) == Some(&world::URBAN) || r.mix_b.get(off) == Some(&world::URBAN);
        if urban && market_of(r, k, c) == Some(t) || r.header.towns.get(t).is_some_and(|tw| tw.node == c) {
            let (x, z) = (c % r.w, c / r.w);
            (x0, z0, x1, z1) = (x0.min(x), z0.min(z), x1.max(x), z1.max(z));
            any = true;
        }
    }
    if !any {
        return Vec::new();
    }
    // a cell round them, since a parcel's ground may be decided by the cell next to it
    let (x0, z0, x1, z1) = (x0.saturating_sub(1), z0.saturating_sub(1), (x1 + 1).min(r.w - 1), (z1 + 1).min(r.h - 1));
    let (nx, nz) = ((x1 + 1 - x0) * PER + 1, (z1 + 1 - z0) * PER + 1);
    let at = |i: usize, j: usize| (x0 as f32 + i as f32 * step, z0 as f32 + j as f32 * step);
    let mut is = vec![false; nx * nz];
    for j in 1..nz - 1 {
        for i in 1..nx - 1 {
            let (x, z) = at(i, j);
            is[j * nx + i] = town_ground_at(r, g, k, x, z) == Some(t);
        }
    }
    trace(&is, nx, nz).into_iter().filter(|l| l.len() >= 8).map(|l| {
        let pts: Vec<(f32, f32)> = l.into_iter().map(|(i, j)| (x0 as f32 + i * step, z0 as f32 + j * step)).collect();
        chaikin(&chaikin(&pts))
    }).collect()
}

/// The edges between the cells that are in and those that are out on a grid
/// `nx` by `nz` whose rim is out, as closed lines in the grid's own units:
/// marching squares, each line through the middles of the sides it crosses.
fn trace(is: &[bool], nx: usize, nz: usize) -> Vec<Vec<(f32, f32)>> {
    // a side of the grid's squares: 2 * corner + 0 for the side running east from it, + 1 south
    let east = |i: usize, j: usize| 2 * (j * nx + i);
    let south = |i: usize, j: usize| 2 * (j * nx + i) + 1;
    let mut links: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut link = |a: usize, b: usize| {
        links.entry(a).or_default().push(b);
        links.entry(b).or_default().push(a);
    };
    for j in 0..nz - 1 {
        for i in 0..nx - 1 {
            let (a, b, c, d) = (is[j * nx + i], is[j * nx + i + 1], is[(j + 1) * nx + i + 1], is[(j + 1) * nx + i]);
            let (top, right, bottom, left) = (east(i, j), south(i + 1, j), east(i, j + 1), south(i, j));
            let crossed: Vec<usize> = [(top, a != b), (right, b != c), (bottom, d != c), (left, a != d)].iter().filter(|e| e.1).map(|e| e.0).collect();
            match crossed.len() {
                2 => link(crossed[0], crossed[1]),
                4 if a => {
                    link(top, left);
                    link(right, bottom);
                }
                4 => {
                    link(top, right);
                    link(left, bottom);
                }
                _ => {}
            }
        }
    }
    let point = |e: usize| {
        let (corner, south) = (e / 2, e % 2 == 1);
        let (i, j) = ((corner % nx) as f32, (corner / nx) as f32);
        if south { (i, j + 0.5) } else { (i + 0.5, j) }
    };
    let mut seen: std::collections::HashSet<usize> = Default::default();
    let mut keys: Vec<usize> = links.keys().copied().collect();
    keys.sort_unstable();
    let mut out = Vec::new();
    for start in keys {
        if seen.contains(&start) {
            continue;
        }
        let mut line = vec![point(start)];
        seen.insert(start);
        let (mut prev, mut at) = (usize::MAX, start);
        loop {
            let Some(next) = links.get(&at).and_then(|n| n.iter().copied().find(|&e| e != prev && !seen.contains(&e))) else { break };
            line.push(point(next));
            seen.insert(next);
            (prev, at) = (at, next);
        }
        out.push(line);
    }
    out
}

/// A closed line with its corners cut: each side gives the points a quarter
/// and three quarters along it.
fn chaikin(pts: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let n = pts.len();
    (0..n)
        .flat_map(|i| {
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            [(a.0 * 0.75 + b.0 * 0.25, a.1 * 0.75 + b.1 * 0.25), (a.0 * 0.25 + b.0 * 0.75, a.1 * 0.25 + b.1 * 0.75)]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The made-up island (`world::tests::island`) with a town, Ashby, whose
    /// market place is at the middle of cell 5, 5 and whose ground is the
    /// three cells 4..6 of row 5, every cell's market Ashby's.
    fn with_a_town() -> Replay {
        let mut r = crate::world::tests::island();
        let node = 5 * r.w + 5;
        r.header.towns = vec![serde_json::from_value(serde_json::json!({"name": "Ashby", "node": node, "founded": 1700})).unwrap()];
        r.n_towns = 1;
        r.town_active = vec![1];
        r.town_pop = vec![5000.0];
        r.town_urban = vec![1200.0];
        r.catch = r.terrain.iter().map(|&t| if t > 0 { 1 } else { 0 }).collect();
        for x in 4..7 {
            let c = 5 * r.w + x;
            (r.mix_a[c], r.mix_b[c], r.mix_s[c]) = (world::URBAN, world::URBAN, 255);
        }
        r
    }

    fn centres(r: &Replay) -> Vec<(f32, f32)> {
        (0..r.n).map(|c| ((c % r.w) as f32 + 0.5, (c / r.w) as f32 + 0.5)).collect()
    }

    #[test]
    fn a_click_picks_a_town_by_all_its_ground_and_open_country_by_the_spot() {
        let r = with_a_town();
        let g = world::build_ground(&r);
        let slots = scatter::make_slots(&r, &g);
        let cen = centres(&r);
        // the town's middle cell, and its far edge, more than 700 m from the market place
        let none = Near::default();
        assert_eq!(choose(&r, &g, &slots, &cen, &none, 5.5, 5.5, 0), Some(Pick::Town(0)));
        let far = (0..PER_TEST).map(|i| 4.05 + 0.02 * i as f32).find(|&x| town_ground_at(&r, &g, 0, x, 5.5) == Some(0) && (x - 5.5).abs() > MARKET_REACH);
        let far = far.expect("some of the town's ground lies more than 700 m from its market place");
        assert_eq!(choose(&r, &g, &slots, &cen, &none, far, 5.5, 0), Some(Pick::Town(0)));
        // open country two cells north: a spot, with the cell under it
        match choose(&r, &g, &slots, &cen, &none, 5.25, 2.75, 0) {
            Some(Pick::Place { x, z, cell }) => assert_eq!((x, z, cell), (5.25, 2.75, 2 * r.w + 5)),
            other => panic!("{other:?}"),
        }
        // the sea, and off the map
        assert_eq!(choose(&r, &g, &slots, &cen, &none, 0.5, 0.5, 0), None);
        assert_eq!(choose(&r, &g, &slots, &cen, &none, -1.0, 3.0, 0), None);
        // a pit within its reach, before the town's ground under it
        let pit = PitSpot { x: 5.5, z: 5.52, cell: 5 * r.w + 5, class: world::COALMINE };
        let with_pit = Near { pits: vec![pit], ..Near::default() };
        assert_eq!(choose(&r, &g, &slots, &cen, &with_pit, 5.5, 5.5, 0), Some(Pick::Pit { cell: pit.cell, class: world::COALMINE }));
        assert_eq!(at_cell(2 * r.w + 5, r.w), Pick::Place { x: 5.5, z: 2.5, cell: 2 * r.w + 5 });
    }

    const PER_TEST: usize = 40;

    #[test]
    fn the_country_s_card_names_its_town_and_ends_with_the_spot() {
        let r = with_a_town();
        let cen = centres(&r);
        let said = describe_place(&r, &cen, 5.5, 2.5, 2 * r.w + 5, 0);
        let (title, body) = said.split_once('\n').unwrap();
        assert_eq!(title, "The country round Ashby");
        assert!(body.contains("5,000 people live in its market area, 1,200 of them in the town, in 1750"), "{body}");
        assert!(body.contains("Round here: 60% farmland and 40% woodland in 1750."), "{body}");
        assert!(body.contains("6.0 km north of Ashby's market place, 11 km east and 5 km south of the island's north-west corner."), "{body}");
        for word in ["square", "cell"] {
            assert!(!said.contains(word), "{said}");
        }
        assert_eq!(bearing(1.0, -1.0), "north-east");
        assert_eq!(bearing(-1.0, 0.1), "west");
        assert!(describe_pit(&r, &cen, 2 * r.w + 5, world::QUARRY, 0).starts_with("Quarry near Ashby\nNo longer worked in 1750."));
    }

    #[test]
    fn a_town_s_outline_is_closed_lines_round_its_ground() {
        let r = with_a_town();
        let g = world::build_ground(&r);
        let lines = town_outline(&r, &g, 0, 0);
        assert!(!lines.is_empty());
        // round the ground: every point within the cells about the town
        for l in &lines {
            for &(x, z) in l {
                assert!((2.5..8.5).contains(&x) && (3.5..7.5).contains(&z), "{x}, {z}");
            }
        }
        // a square of four looks in, traced and closed
        let mut is = vec![false; 16];
        for (i, j) in [(1, 1), (2, 1), (1, 2), (2, 2)] {
            is[j * 4 + i] = true;
        }
        let t = trace(&is, 4, 4);
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].len(), 8);
    }
}
