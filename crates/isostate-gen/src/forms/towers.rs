//! The tall forms of the "tall" direction, built again of the houses' kit:
//! the office block of brick and terracotta of the first steel frames, the
//! stepped tower in stone of between the wars, and the tower of flats of the
//! new century.
//!
//! The designer of that direction drew them as boxes with grids of strips
//! for windows, well below the houses' fineness. Here each is walls of the
//! kit with its windows let into them a storey at a time, as a house's are,
//! so that close to a window has its reveal, its sill and its bars; a base
//! that meets the street (shop fronts, a door you can see is a door, stone
//! to the first floor); and a top that ends (a cornice and a water tank, a
//! crown and a spire, a storey set back under its plant). The windows are
//! the decade's: sashes in pairs to 1929, tall metal casements in slots
//! between piers after, wide casements and glass balconies from 1986.
//!
//! What a tower is handed is what the simulation would know of its lot: the
//! ground (`w`, `d`), the storeys its rent asks for, and the year it went up.

use std::f32::consts::FRAC_PI_4;

use glam::Vec3;

use super::{flat_roof, rows, thick};
use crate::colour::{pick, shade, Rgba};
use crate::hash::Seed;
use crate::kit::{Detail, Kit, Open, Opening, Over, Range, Side, Wall, FOOT, STOREY};
use crate::palette::Palette;
use crate::site::{Model, Site};

#[derive(Clone, Copy, Debug)]
pub struct Tower {
    /// the lot: along the street and back from it
    pub w: f32,
    pub d: f32,
    pub storeys: u8,
    /// the year it went up
    pub built: i32,
    pub detail: Detail,
    pub seed: Seed,
}

/// How far in from the lot's edge a tower's walls stand.
const MARGIN: f32 = 0.004;
/// How far apart a tower's windows stand along its walls.
const PITCH: f32 = 0.0135;

// (Walls that begin above the first floor are built with `Side::free`: the
// kit builds a `Side::with` wall a storey at a time counted from the ground,
// which is right for a wall that stands on it and not for one that stands on
// another.)
fn side_of(tier: usize, ops: &[Opening]) -> Side<'_> {
    if tier == 0 { Side::with(ops) } else { Side::free(ops) }
}

/// The ground floor of a tower on its street: shop fronts along the front,
/// a door to the upper floors in its middle, windows round its other sides.
fn street_floor(len: f32, dep: f32, paints: &[Rgba; 2], awning: Option<Rgba>, door: Open) -> (Vec<Opening>, Vec<Opening>) {
    let bays = ((len / (2.0 * PITCH)).floor() as usize).max(2);
    let mut front = Vec::new();
    for b in 0..bays {
        let u = len * (b as f32 + 0.5) / bays as f32;
        if b == bays / 2 {
            front.push(Opening { u, y: FOOT, w: 0.0094, h: 0.0152, kind: door });
        } else {
            front.push(Opening { u, y: FOOT, w: len / bays as f32 * 0.78, h: 0.0134, kind: Open::Shop { paint: paints[b % 2], awning } });
        }
    }
    let side = rows(dep, 0, 1, 0.0072, 0.0092, 0.0058, Open::Sash, PITCH);
    (front, side)
}

/// An office block of the first steel frames, 1890 to 1929: five to twelve
/// storeys of brick or buff terracotta on two of stone, sashes in pairs to
/// every bay, a cornice that oversails, and a water tank on the roof.
pub(crate) fn office_block(m: &mut Model, site: &Site, t: &Tower, pal: &Palette) {
    let r = |n: i32| t.seed.pick(n);
    let mut k = Kit::new(&mut m.solid, site, pal, t.detail, 0.0);
    let (hw, hd) = (t.w * 0.5 - MARGIN, t.d * 0.5 - MARGIN);
    let n = t.storeys.max(3) as usize;
    let h = n as f32 * STOREY;
    let rg = Range::rect(-hw, hw, -hd, hd);
    let y0 = -k.drop(&rg);
    let body = [pal.brick.terrace(r(1)), *pal.walls.ochre, *pal.brick.red, *pal.modern.portland][(r(2) * 4.0) as usize % 4];
    let stone = *pal.modern.portland;
    let base = Wall { col: stone, plinth: Some(shade(stone, 0.8)), frame: *pal.walls.window_frame, sill: shade(stone, 1.04) };
    let upper = Wall { col: body, plinth: None, frame: *pal.walls.window_frame, sill: stone };
    let paints = [pick(&pal.walls.door_paints, r(3)), pick(&pal.walls.door_paints, r(4))];
    let awning = (r(5) < 0.6).then(|| pick(&pal.roofs.awnings, r(6)));
    let (len, dep) = (rg.len, rg.dep);
    // the base: shops and the door on the ground floor, tall windows of the banking floor over them
    let (mut front, mut side) = if k.far() { (Vec::new(), Vec::new()) } else { street_floor(len, dep, &paints, awning, Open::Door { paint: paints[0], over: Over::Fanlight }) };
    let mut back = Vec::new();
    if !k.far() {
        front.extend(rows(len, 1, 2, 0.0072, 0.0118, 0.0044, Open::Sash, PITCH));
        side.extend(rows(dep, 1, 2, 0.0072, 0.0118, 0.0044, Open::Sash, PITCH));
        back = rows(len, 0, 2, 0.0066, 0.0094, 0.006, Open::Sash, PITCH);
    }
    k.walls(&rg, y0, 2.0 * STOREY, &base, [Side::with(&front), Side::with(&side), Side::with(&back), Side::with(&side)], true, false);
    // the floors of offices: a pair of sashes to every bay, one under the other bay's piers
    let (long, short) = if k.far() { (Vec::new(), Vec::new()) } else { (rows(len, 2, n, 0.0062, 0.0098, 0.0056, Open::Sash, PITCH * 0.62), rows(dep, 2, n, 0.0062, 0.0098, 0.0056, Open::Sash, PITCH * 0.62)) };
    k.walls(&rg, 2.0 * STOREY, h, &upper, [Side::free(&long), Side::free(&short), Side::free(&long), Side::free(&short)], false, true);
    // a cornice over the base and another, deeper, over the top, oversailing the walls
    let c = 0.0024;
    k.boxy(-hw - c, hw + c, -hd - c, hd + c, h - 0.0018, h + 0.0024, shade(stone, 0.96), Some(shade(stone, 1.04)));
    flat_roof(&mut k, -hw, hw, -hd, hd, h + 0.0024, 0.0034, shade(body, 0.94), *pal.modern.gravel);
    if !k.far() {
        k.boxy(-hw - 0.0012, hw + 0.0012, -hd - 0.0012, hd + 0.0012, 2.0 * STOREY - 0.0008, 2.0 * STOREY + 0.0012, shade(stone, 1.02), Some(shade(stone, 1.06)));
        // piers of brick between the pairs of sashes, the frame showing through the wall
        for side in 0..4 {
            let l = rg.face(side).len;
            let bays = ((l / (PITCH * 0.62)).floor() as usize).max(2);
            for b in (0..=bays).step_by(2) {
                let u = (l * b as f32 / bays as f32).clamp(0.0008, l - 0.0008);
                k.strip(&rg, side, u - 0.0008, u + 0.0008, 2.0 * STOREY + 0.001, h - 0.002, 0.0007, shade(body, 1.06));
            }
        }
    }
    // a water tank of staves on its legs, and the lift's motor room
    let (x, z) = (hw * 0.5 * (r(7) - 0.5) * 2.0, -hd * 0.35);
    k.boxy(-hw * 0.3, hw * 0.1, 0.0, hd * 0.6, h + 0.0024, h + 0.0024 + STOREY * 0.8, shade(body, 0.9), Some(*pal.modern.gravel));
    if !k.far() {
        let p = k.pt(x, h + 0.0024, z);
        let wood = *pal.timber.boards;
        k.solid.block(p, 0.008, 0.008, 0.004, site.yaw, *pal.iron.plate, None, 0.0);
        k.solid.frustum(p + Vec3::Y * 0.004, 0.0058, 0.0058, 0.0085, 8, wood, None);
        k.solid.frustum(p + Vec3::Y * 0.0125, 0.0062, 0.0, 0.004, 8, shade(wood, 0.8), None);
    }
}

/// A height held to a whole number of storeys.
fn snap(y: f32) -> f32 {
    (y / STOREY).round() * STOREY
}

/// A stepped tower of between the wars, 1925 to 1959: three tiers of pale
/// stone, each set back from the one below as a zoning law would ask, tall
/// metal windows stacked in slots between piers that run clear to the top
/// of each tier, a base of shops and a bronze door, and a crown of two short
/// stages under a copper pyramid or a spire.
pub(crate) fn stepped(m: &mut Model, site: &Site, t: &Tower, pal: &Palette) {
    let r = |n: i32| t.seed.pick(n);
    let mut k = Kit::new(&mut m.solid, site, pal, t.detail, 0.0);
    let (hw, hd) = (t.w * 0.5 - MARGIN, t.d * 0.5 - MARGIN);
    let n = t.storeys.max(6) as usize;
    let h = n as f32 * STOREY;
    let body = [*pal.modern.portland, *pal.walls.cream, *pal.stone.limestone, *pal.brick.pale][(r(1) * 4.0) as usize % 4];
    let wall = Wall { col: body, plinth: Some(shade(body, 0.8)), frame: *pal.modern.mullion, sill: shade(body, 1.05) };
    let paints = [*pal.modern.mullion, pick(&pal.walls.door_paints, r(3))];
    let window = Open::Paned { cols: 2, rows: 3, round: false, lit: false };
    // the tiers: each to a whole storey, each inset from the one below
    let cuts = [0.0, snap(h * (0.52 + 0.1 * r(2))), snap(h * (0.78 + 0.06 * r(3))), h];
    let insets = [0.0, 0.16, 0.3];
    for i in 0..3 {
        let (ya, yb) = (cuts[i], cuts[i + 1]);
        if yb <= ya + 0.5 * STOREY {
            continue;
        }
        let (w, d) = (hw * (1.0 - insets[i]), hd * (1.0 - insets[i]));
        let rg = Range::rect(-w, w, -d, d);
        let (s0, s1) = ((ya / STOREY).round() as usize, (yb / STOREY).round() as usize);
        let y0 = if i == 0 { -k.drop(&rg) } else { ya };
        let mut long = Vec::new();
        let mut short = Vec::new();
        if !k.far() {
            if i == 0 {
                let (f, s) = street_floor(rg.len, rg.dep, &paints, None, Open::Leaves { paint: *pal.modern.mullion, round: true });
                long = f;
                short = s;
            }
            let from = if i == 0 { 1 } else { s0 };
            long.extend(rows(rg.len, from, s1, 0.0058, 0.0124, 0.0042, window, PITCH));
            short.extend(rows(rg.dep, from, s1, 0.0058, 0.0124, 0.0042, window, PITCH));
        }
        let back: Vec<Opening> = if k.far() { Vec::new() } else { rows(rg.len, if i == 0 { 1 } else { s0 }, s1, 0.0058, 0.0124, 0.0042, window, PITCH) };
        k.walls(&rg, y0, yb, &wall, [side_of(i, &long), side_of(i, &short), side_of(i, &back), side_of(i, &short)], i == 0, false);
        if !k.far() {
            // the piers between the slots, proud of the wall and paler, clear to the tier's top
            for side in 0..4 {
                let l = rg.face(side).len;
                let bays = ((l / PITCH).floor() as usize).max(2);
                for b in 0..=bays {
                    let u = (l * b as f32 / bays as f32).clamp(0.0009, l - 0.0009);
                    k.strip(&rg, side, u - 0.0009, u + 0.0009, ya.max(STOREY) + 0.001, yb - 0.0006, 0.0009, shade(body, 1.08));
                }
                if k.near() {
                    // the dark spandrels under the windows, a band to each storey
                    for s in s0.max(1)..s1 {
                        let y = s as f32 * STOREY;
                        for b in 0..bays {
                            let u = l * (b as f32 + 0.5) / bays as f32;
                            k.strip(&rg, side, u - 0.0029, u + 0.0029, y + 0.0004, y + 0.0036, 0.0004, shade(*pal.modern.mullion, 1.3));
                        }
                    }
                }
            }
        }
        // a coping at each setback
        k.boxy(-w - 0.0010, w + 0.0010, -d - 0.0010, d + 0.0010, yb - 0.0014, yb + 0.0010, shade(body, 1.06), Some(*pal.modern.gravel));
    }
    // the crown: two short stages, then a copper pyramid or a spire
    let (w, d) = (hw * 0.58, hd * 0.58);
    let c1 = h + STOREY * 1.5;
    let crg = Range::rect(-w, w, -d, d);
    let louvres: Vec<Opening> = if k.far() { Vec::new() } else { rows(crg.len, 0, 1, 0.0046, 0.016, 0.0, Open::Louvre, PITCH * 1.4).into_iter().map(|o| Opening { y: h + 0.0045, ..o }).collect() };
    let louvres_d: Vec<Opening> = if k.far() { Vec::new() } else { rows(crg.dep, 0, 1, 0.0046, 0.016, 0.0, Open::Louvre, PITCH * 1.4).into_iter().map(|o| Opening { y: h + 0.0045, ..o }).collect() };
    k.walls(&crg, h, c1, &Wall { col: shade(body, 1.04), ..wall }, [Side::free(&louvres), Side::free(&louvres_d), Side::free(&louvres), Side::free(&louvres_d)], false, false);
    k.boxy(-w - 0.001, w + 0.001, -d - 0.001, d + 0.001, c1 - 0.0012, c1 + 0.001, shade(body, 1.1), Some(*pal.modern.gravel));
    let (w2, d2) = (w * 0.66, d * 0.66);
    let c2 = c1 + STOREY;
    k.boxy(-w2, w2, -d2, d2, c1, c2, shade(body, 1.08), Some(*pal.modern.gravel));
    let p = k.pt(0.0, c2, 0.0);
    if r(4) < 0.5 {
        k.solid.frustum_turned(p, w2.min(d2) * 1.2, 0.0, STOREY * 3.0, 4, FRAC_PI_4 - site.yaw, 0.0, *pal.modern.copper, None);
        k.solid.frustum(p + Vec3::Y * STOREY * 2.6, 0.0013, 0.0, STOREY * 2.2, 4, *pal.modern.copper, None);
    } else {
        k.solid.frustum_turned(p, w2.min(d2) * 0.9, w2.min(d2) * 0.5, STOREY * 1.5, 8, 0.0, 0.0, shade(body, 1.08), None);
        k.solid.frustum(p + Vec3::Y * STOREY * 1.5, 0.0024, 0.0, STOREY * 5.0, 4, *pal.stone.church_spire, None);
    }
}

/// A tower of flats of the new century, from 1986: ten storeys or more in
/// render of one of the town's washes, wide casements to every bay, a
/// balcony with a rail of glass to every other bay on the front and the
/// back, a glazed hall at its foot behind a canopy, and the top storey set
/// back in boards under the lift's motor room.
pub(crate) fn flats(m: &mut Model, site: &Site, t: &Tower, pal: &Palette) {
    let r = |n: i32| t.seed.pick(n);
    let mut k = Kit::new(&mut m.solid, site, pal, t.detail, 0.0);
    let (hw, hd) = (t.w * 0.5 - MARGIN - 0.004, t.d * 0.5 - MARGIN - 0.006);
    let n = t.storeys.max(4) as usize;
    let top = n - 1;
    let (h, ht) = (top as f32 * STOREY, n as f32 * STOREY);
    let body = pal.walls.town_house(r(1));
    let concrete = *pal.modern.concrete;
    let wall = Wall { col: body, plinth: Some(shade(concrete, 0.82)), frame: *pal.walls.window_frame, sill: concrete };
    let rg = Range::rect(-hw, hw, -hd, hd);
    let y0 = -k.drop(&rg);
    let (len, dep) = (rg.len, rg.dep);
    // the hall at its foot: a wide glazed front and a door, windows round the rest
    let mut front = Vec::new();
    let mut side = Vec::new();
    let mut back = Vec::new();
    if !k.far() {
        front.push(Opening { u: len * 0.5, y: FOOT, w: 0.03, h: 0.0152, kind: Open::Paned { cols: 5, rows: 2, round: false, lit: false } });
        front.extend(rows(len, 1, top, 0.0098, 0.0104, 0.0048, Open::Casement, PITCH).into_iter());
        for o in rows(len, 0, 1, 0.0098, 0.0092, 0.006, Open::Casement, PITCH) {
            if (o.u - len * 0.5).abs() > 0.022 {
                front.push(o);
            }
        }
        side = rows(dep, 0, top, 0.0098, 0.0104, 0.0048, Open::Casement, PITCH);
        back = rows(len, 0, top, 0.0098, 0.0104, 0.0048, Open::Casement, PITCH);
    }
    k.walls(&rg, y0, h, &wall, [Side::with(&front), Side::with(&side), Side::with(&back), Side::with(&side)], true, false);
    if !k.far() {
        // a band of concrete at the first floor, where the hall's front ends
        k.boxy(-hw - 0.0008, hw + 0.0008, -hd - 0.0008, hd + 0.0008, STOREY - 0.0008, STOREY + 0.0008, concrete, Some(shade(concrete, 1.06)));
        // the canopy over the door
        k.boxy(-0.018, 0.018, hd, hd + 0.008, 0.0165, 0.0178, concrete, Some(shade(concrete, 1.05)));
        // balconies: a slab and a rail of glass, to every other bay, front and back, from the second storey
        let bays = ((len / PITCH).floor() as usize).max(2);
        // a rail of painted metal to 2000, of glass after
        let rail = if t.built < 2000 { pick(&pal.modern.panels, r(5)) } else { *pal.modern.tower_glass };
        for s in 1..top {
            let y = s as f32 * STOREY;
            for b in ((s + (r(4) * 2.0) as usize) % 2..bays).step_by(2) {
                let x = -hw + len * (b as f32 + 0.5) / bays as f32;
                for (z0, z1, zf) in [(hd, hd + 0.0044, hd + 0.0044), (-hd - 0.0044, -hd, -hd - 0.0044)] {
                    k.boxy(x - 0.0058, x + 0.0058, z0, z1, y - thick(&k, 0.0012) * 0.5, y + thick(&k, 0.0012) * 0.5, shade(concrete, 0.96), Some(concrete));
                    // (as thin as glass close to; from the play distance as thick as the level may draw)
                    let half = thick(&k, 0.0006) * 0.5;
                    k.boxy(x - 0.0058, x + 0.0058, zf - half, zf + half, y + 0.0006, y + 0.0058, rail, None);
                }
            }
        }
    }
    flat_roof(&mut k, -hw, hw, -hd, hd, h, 0.0028, shade(body, 0.95), *pal.modern.gravel);
    // the top storey, set back from the front, in boards with wide glass and a terrace before it
    let back_by = 0.012;
    let trg = Range::rect(-hw + 0.004, hw - 0.004, -hd + 0.002, hd - back_by);
    let boards = Wall { col: *pal.modern.cladding_timber, plinth: None, frame: *pal.modern.mullion, sill: concrete };
    let (tf, ts) = if k.far() { (Vec::new(), Vec::new()) } else { (rows(trg.len, top, n, 0.0108, 0.0128, 0.003, Open::Casement, PITCH), rows(trg.dep, top, n, 0.0072, 0.0104, 0.0048, Open::Casement, PITCH)) };
    k.walls(&trg, h, ht, &boards, [Side::free(&tf), Side::free(&ts), Side::free(&ts), Side::free(&ts)], false, false);
    flat_roof(&mut k, trg.cx - trg.len * 0.5 - 0.002, trg.cx + trg.len * 0.5 + 0.002, -hd, hd - back_by + 0.003, ht, 0.0, *pal.modern.mullion, *pal.modern.green_roof);
    // the lift's motor room, and the rail round the terrace
    k.boxy(-hw * 0.25, hw * 0.25, -hd * 0.45, hd * 0.1, ht, ht + STOREY * 0.9, shade(body, 0.92), Some(*pal.modern.gravel));
    if !k.far() {
        k.boxy(-hw + 0.001, hw - 0.001, hd - 0.0006 - thick(&k, 0.0012), hd - 0.0006, h + 0.0006, h + 0.006, *pal.modern.tower_glass, None);
    }
}
