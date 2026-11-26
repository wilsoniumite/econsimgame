//! Where the pits stand, under rules that keep the quarries, pits and mines as
//! workings (the core's docs/rules/mines.md).
//!
//! Until 26 November 2026 each working took the first free of four fixed
//! spots in its cell, on a jittered grid of two by two, so a coalfield was a
//! lattice of pits about two kilometres apart, each near the same quarter of
//! its own cell. Now a pit stands where its working and the ground put it.
//! Its cell offers sixteen spots, laid at random over it and kept off the
//! beach and 50 m clear of the rivers (`ground_spots`), of which a working
//! takes the one that scores best:
//!
//! - over its deposit: the richness of its kind of ground, laid between the
//!   middles of the cells, is the higher there than at its cell's middle;
//! - by the way that serves it: on the side of its cell toward the link the
//!   most was carried over, out of its cell or into it, in the year it opened
//!   (the record's traffic; a run without traffic leaves this out);
//! - a draw of its own, so that no two cells put their pits alike;
//! - and before all of these, the ground its square paints as its kind of
//!   diggings that year, where any of its spots lies on it.
//!
//! It never takes a spot on its cell's town ground, works ground or derelict
//! ground in the year it opened, on the plot of a works standing then, or
//! within 240 m of a pit of its cell placed before it. Everything it is
//! placed by is of the year it opened, so its place is fixed that year and
//! the same whichever year is looked at first, in a replay and in a live
//! game alike; it never moves after. A works' yard laid over it in a later
//! year hides it that year (`scatter::build_year`), as before.
//!
//! A working is known by its cell, its kind, the year it was opened and its
//! number among workings alike in all three (`Key`). The places are worked
//! out once and kept (`Slots::pits`).

use std::collections::HashMap;
use std::sync::Mutex;

use super::{grid_slots, Slot, Slots};
use crate::replay::{self, Replay, Working};
use crate::world::{self, Ground};

/// How close to a river's middle a pit may stand, in cells (50 m).
const CLEAR_WATER: f32 = 0.025;
/// How close two pits of one cell may stand, in cells (240 m).
const APART: f32 = 0.12;
/// The weights of the three things a spot is scored by.
const BY_DEPOSIT: f32 = 1.2;
const BY_WAY: f32 = 0.8;
const BY_DRAW: f32 = 0.9;
/// What standing on its own kind of diggings counts for: more than the other
/// three together, so a pit stands on the ground painted for it where it can.
const BY_GROUND: f32 = 3.0;

/// The spots a pit may take: sixteen a cell, laid at random over it, sorted
/// by the warped cell whose ground each lies on (`world::class_key`), as the
/// mine slots were.
pub fn ground_spots(r: &Replay, g: &Ground) -> Vec<Slot> {
    let mut v = grid_slots(r, g, 4, 23, 1.0, false);
    v.retain(|s| world::water_dist(g, s.x, s.z) >= 2.0 * CLEAR_WATER);
    v.sort_by_key(|s| s.wcell);
    v
}

/// A working, as the pits know it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Key {
    pub opened: i32,
    pub cell: u32,
    pub class: u8,
    /// its number among the workings of the year's list alike in the three above
    pub twin: u16,
}

/// Where a pit stands, and the two numbers it is drawn with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spot {
    pub x: f32,
    pub z: f32,
    pub yaw: f32,
    pub v: f32,
}

/// A pit standing in a year.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Standing {
    pub at: Spot,
    pub cell: usize,
    pub class: u8,
}

/// The places worked out so far, kept for the run. Shared by every copy of
/// the slots, as the works' plans are.
#[derive(Default)]
pub struct Places(Mutex<HashMap<Key, Option<Spot>>>);

impl Places {
    /// Forget every place, for a history taken back by hand (`back.rs`).
    pub fn forget(&self) {
        if let Ok(mut m) = self.0.lock() {
            m.clear();
        }
    }
}

fn is_pit(class: u8) -> bool {
    (world::QUARRY..=world::COALMINE).contains(&class)
}

/// The key of each working of a year's list, None for one that is no pit.
pub fn keys(list: &[Working]) -> Vec<Option<Key>> {
    let mut seen: HashMap<(i32, u32, u8), u16> = HashMap::new();
    list.iter()
        .map(|w| {
            if w.opened == 0 || !is_pit(w.class) {
                return None;
            }
            let n = seen.entry((w.opened, w.cell, w.class)).or_insert(0);
            let key = Key { opened: w.opened, cell: w.cell, class: w.class, twin: *n };
            *n += 1;
            Some(key)
        })
        .collect()
}

/// The pits standing in year `k`: every working at work, each where it was
/// placed the year it opened. A spot covered this year by a works' yard is
/// for the caller to leave out.
pub fn standing(r: &Replay, slots: &Slots, k: usize) -> Vec<Standing> {
    let Some(list) = r.workings.get(k).filter(|w| !w.is_empty()) else { return Vec::new() };
    let firms = firms_by_cell(r, slots);
    let mut known = slots.pits.0.lock().unwrap();
    let mut out = Vec::new();
    for (w, key) in list.iter().zip(keys(list)) {
        let Some(key) = key else { continue };
        if w.hands < 1.0 {
            continue;
        }
        if let Some(at) = place(r, slots, &firms, &mut known, key) {
            out.push(Standing { at, cell: w.cell as usize, class: w.class });
        }
    }
    out
}

fn firms_by_cell(r: &Replay, slots: &Slots) -> HashMap<usize, Vec<usize>> {
    let mut by: HashMap<usize, Vec<usize>> = HashMap::new();
    for (f, firm) in r.header.firms.iter().enumerate().take(slots.firm_pos.len().min(slots.firm_plot.len())) {
        by.entry(firm.cell).or_default().push(f);
    }
    by
}

/// The year index a working opened in, kept within the years the run holds.
fn opened_k(r: &Replay, key: Key) -> usize {
    let last = r.filled.min(r.ny).saturating_sub(1);
    ((key.opened - r.y0).max(0) as usize).min(last)
}

/// Where the working `key` stands: worked out the first time it is asked
/// for, from the island as it stood the year it opened, and kept.
fn place(r: &Replay, slots: &Slots, firms: &HashMap<usize, Vec<usize>>, known: &mut HashMap<Key, Option<Spot>>, key: Key) -> Option<Spot> {
    if let Some(&s) = known.get(&key) {
        return s;
    }
    let k0 = opened_k(r, key);
    // the pits of its cell that stood the year it opened and were placed before it
    let before: Vec<Key> = r.workings.get(k0).map_or(Vec::new(), |list| keys(list).into_iter().flatten().filter(|b| b.cell == key.cell && *b < key).collect());
    let taken: Vec<Spot> = before.into_iter().filter_map(|b| place(r, slots, firms, known, b)).collect();
    let spot = choose(r, slots, firms, key, k0, &taken);
    known.insert(key, spot);
    spot
}

/// The richness of a pit's kind of ground at a spot, laid between the
/// middles of the cells.
fn deposit_at(r: &Replay, field: &[u8], x: f32, z: f32) -> f32 {
    let (bx, bz) = (x - 0.5, z - 0.5);
    let (ix, iz) = (bx.floor(), bz.floor());
    let (fx, fz) = (bx - ix, bz - iz);
    let at = |dx: i32, dz: i32| {
        let (cx, cz) = ((ix as i32 + dx).clamp(0, r.w as i32 - 1) as usize, (iz as i32 + dz).clamp(0, r.h as i32 - 1) as usize);
        field.get(cz * r.w + cx).map_or(0.0, |&b| b as f32 / 255.0)
    };
    let top = at(0, 0) + (at(1, 0) - at(0, 0)) * fx;
    let bottom = at(0, 1) + (at(1, 1) - at(0, 1)) * fx;
    top + (bottom - top) * fz
}

/// The way most was carried out of cell `c` or into it in year `k`, as a
/// direction on the map, or nothing where the run kept no traffic or nothing went.
fn busiest_way(r: &Replay, c: usize, k: usize) -> Option<(f32, f32)> {
    use isostate_core::traffic::{other_end, SLOTS};
    if !r.traffic.kept() || k >= r.filled.min(r.ny) {
        return None;
    }
    let mut best: Option<(f32, (f32, f32))> = None;
    let (x, y) = ((c % r.w) as isize, (c / r.w) as isize);
    for (s, &(dx, dy)) in SLOTS.iter().enumerate() {
        // out of c in slot s, and into c from the cell whose slot s ends at c
        let out = r.traffic.weight(r.n, k, c, s);
        let (mx, my) = (x - dx, y - dy);
        let back = if mx >= 0 && my >= 0 && (mx as usize) < r.w && (my as usize) < r.h {
            let m = my as usize * r.w + mx as usize;
            if other_end(r.w, r.h, m, s) == Some(c) { r.traffic.weight(r.n, k, m, s) } else { 0.0 }
        } else {
            0.0
        };
        for (w, d) in [(out, (dx as f32, dy as f32)), (back, (-dx as f32, -dy as f32))] {
            if w > 0.0 && best.is_none_or(|b| w > b.0) {
                best = Some((w, d));
            }
        }
    }
    best.map(|(_, (dx, dy))| {
        let l = (dx * dx + dy * dy).sqrt();
        (dx / l, dy / l)
    })
}

fn choose(r: &Replay, slots: &Slots, firms: &HashMap<usize, Vec<usize>>, key: Key, k0: usize, taken: &[Spot]) -> Option<Spot> {
    let c = key.cell as usize;
    let spots = &slots.pit_ground;
    let lo = spots.partition_point(|s| s.wcell < key.cell);
    let hi = spots.partition_point(|s| s.wcell <= key.cell);
    if lo == hi {
        return None;
    }
    let field_name = match key.class {
        world::COALMINE => "coal",
        world::IRONMINE => "ore",
        world::CLAYPIT => "clay",
        _ => "rock",
    };
    let field = r.holds.field(field_name);
    let (mx, mz) = ((c % r.w) as f32 + 0.5, (c / r.w) as f32 + 0.5);
    let middle = field.map_or(0.0, |f| deposit_at(r, f, mx, mz));
    let way = busiest_way(r, c, k0);
    let seed = (key.cell as i32).wrapping_mul(31).wrapping_add(key.opened).wrapping_mul(7).wrapping_add(key.class as i32 * 3 + key.twin as i32);
    // the works standing the year it opened, in its cell and round it
    let (cx, cz) = ((c % r.w) as isize, (c / r.w) as isize);
    let mut near_works = Vec::new();
    for dz in -1..=1isize {
        for dx in -1..=1isize {
            let (xx, zz) = (cx + dx, cz + dz);
            if xx < 0 || zz < 0 || xx as usize >= r.w || zz as usize >= r.h {
                continue;
            }
            if let Some(fs) = firms.get(&(zz as usize * r.w + xx as usize)) {
                near_works.extend(fs.iter().copied().filter(|&f| replay::firm_k(r, f, k0) > 0.0));
            }
        }
    }
    let mut best: Option<(f32, usize)> = None;
    for (i, s) in spots[lo..hi].iter().enumerate() {
        let ground = world::pick_class(r, k0, s.wcell as usize, s.sub);
        if matches!(ground, world::URBAN | world::WORKS | world::DERELICT) {
            continue;
        }
        if taken.iter().any(|t| (t.x - s.x).hypot(t.z - s.z) < APART) {
            continue;
        }
        if near_works.iter().any(|&f| crate::pick::on_plot(slots, f, &slots.firm_plot[f], s.x, s.z, 0.03)) {
            continue;
        }
        let rich = field.map_or(0.0, |f| ((deposit_at(r, f, s.x, s.z) - middle) / middle.max(0.05)).clamp(-1.0, 1.0));
        let toward = way.map_or(0.0, |(wx, wz)| 2.0 * ((s.x - mx) * wx + (s.z - mz) * wz));
        let draw = world::hash2(seed, i as i32, 211);
        // the ground its square paints as its diggings, before the fields round it
        let diggings = if ground == key.class { BY_GROUND } else { 0.0 };
        let score = BY_DEPOSIT * rich + BY_WAY * toward + BY_DRAW * draw + diggings;
        if best.is_none_or(|b| score > b.0) {
            best = Some((score, i));
        }
    }
    best.map(|(_, i)| {
        let s = &spots[lo + i];
        Spot { x: s.x, z: s.z, yaw: world::hash2(seed, 1, 213) * std::f32::consts::TAU, v: world::hash2(seed, 2, 213) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn working(cell: u32, class: u8, opened: i32) -> Working {
        Working { cell, class, opened, hands: 10.0, out: 0.0, heap: 0.0, surface: 0.0 }
    }

    /// Three collieries of one square of the made-up island and a quarry
    /// beside it: each stands on its own square's ground, apart from the
    /// others, and where it stands does not hang on the order they are asked
    /// for in, on what was asked before, or on a working closing beside it.
    #[test]
    fn a_pit_stands_where_its_working_puts_it_and_nowhere_else() {
        let mut r = world::tests::island();
        let c = 5 * r.w + 5;
        for m in [c, c + 1] {
            (r.mix_a[m], r.mix_b[m], r.mix_s[m]) = (world::COALMINE, world::COALMINE, 255);
        }
        let list = vec![working(c as u32, world::COALMINE, 1750), working(c as u32, world::COALMINE, 1750), working(c as u32, world::COALMINE, 1748), working(c as u32 + 1, world::QUARRY, 1750)];
        r.workings = vec![list.clone()];
        let g = world::build_ground(&r);
        let slots = super::super::make_slots(&r, &g);
        let first = standing(&r, &slots, 0);
        assert_eq!(first.len(), 4, "every working at work has its pit: {first:?}");
        for (i, a) in first.iter().enumerate() {
            let (wc, _) = world::class_key(&g, a.at.x, a.at.z);
            assert_eq!(wc, a.cell, "a pit stands on its own square's ground: {a:?}");
            for b in &first[i + 1..] {
                if a.cell == b.cell {
                    assert!((a.at.x - b.at.x).hypot(a.at.z - b.at.z) >= APART, "two pits of one square stand apart: {a:?} {b:?}");
                }
            }
        }
        // the places are the same asked afresh, in the other order, and with the
        // youngest colliery closed: none moves
        let mut again = r.clone();
        again.workings = vec![list.iter().rev().cloned().collect()];
        let fresh = super::super::make_slots(&again, &g);
        let second = standing(&again, &fresh, 0);
        for a in &first {
            assert!(second.contains(a), "{a:?} moved when asked in another order: {second:?}");
        }
        let mut closed = list.clone();
        closed[1].hands = 0.0;
        again.workings = vec![closed];
        let third = standing(&again, &super::super::make_slots(&again, &g), 0);
        assert_eq!(third.len(), 3);
        for a in &third {
            assert!(first.contains(a), "{a:?} moved when a working beside it closed");
        }
    }

    #[test]
    fn twins_are_told_apart_and_what_is_no_pit_has_no_key() {
        let list = [working(5, world::COALMINE, 1800), working(5, world::COALMINE, 1800), working(5, world::FARM, 1800), working(6, world::COALMINE, 0)];
        let k = keys(&list);
        assert_eq!(k[0].unwrap().twin, 0);
        assert_eq!(k[1].unwrap().twin, 1);
        assert!(k[2].is_none() && k[3].is_none());
        assert!(k[0].unwrap() < k[1].unwrap());
    }
}
