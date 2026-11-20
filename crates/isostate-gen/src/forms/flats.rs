//! Flats: the council's low flats of after 1945, the slab block and the
//! point block of the 1960s. Built of the kit, a storey and a bay at a
//! time, as every building is: a slab is a long house of many storeys, not
//! a grey box.

use crate::colour::{pick, shade, Rgba};
use super::{band_round, door, flat_roof, rows, thick, win, Tones};
use crate::hash::Seed;
use crate::houses::Detail;
use crate::kit::{End, Kit, Open, Opening, Range, RoofSpec, Side, Wall, FOOT, STOREY};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::scale::BAY;
use crate::site::Site;

/// Low flats: three or four storeys of brick under a hipped roof, the
/// stairs lit by tall windows, balconies on the front. 1945 to 1960.
#[derive(Clone, Copy, Debug)]
pub struct LowFlats {
    /// bays along the front: a flat is two
    pub bays: usize,
    /// three, or four where the ground was dear (the square's `dens`)
    pub storeys: u8,
    pub detail: Detail,
    pub seed: Seed,
}

impl LowFlats {
    pub fn length(&self) -> f32 {
        self.bays as f32 * BAY * 1.04
    }
}

pub const LOW_FLATS_DEPTH: f32 = 0.031;

pub(crate) fn low_flats(solid: &mut MeshBuf, site: &Site, f: &LowFlats, pal: &Palette) {
    let t = Tones::of(pal);
    let r = |n: i32| f.seed.pick(n);
    let mut k = Kit::new(solid, site, pal, f.detail, 0.0);
    let (len, dep) = (f.length(), LOW_FLATS_DEPTH);
    let n = f.storeys as usize;
    let h = n as f32 * STOREY;
    let rg = Range::rect(-len * 0.5, len * 0.5, -dep * 0.5, dep * 0.5);
    let y0 = -k.drop(&rg);
    let brick = pal.brick.terrace(r(1) * 0.7);
    let wall = Wall { col: brick, plinth: Some(shade(brick, 0.8)), frame: *pal.walls.window_frame, sill: t.concrete };
    let paint = pick(&pal.walls.door_paints, r(2));
    let bay = len / f.bays as f32;
    // a stair every four bays, from the second
    let stair = |b: usize| b % 4 == 1;
    let mut front = Vec::new();
    let mut back = Vec::new();
    if !k.far() {
        for b in 0..f.bays {
            let u = bay * (b as f32 + 0.5);
            for s in 0..n {
                if stair(b) {
                    if s == 0 {
                        front.push(door(u, paint));
                    }
                    // the stair's tall window between floors
                    front.push(Opening { u, y: s as f32 * STOREY + 0.011, w: 0.0052, h: 0.0124, kind: Open::Paned { cols: 1, rows: 3, round: false, lit: false } });
                } else {
                    front.push(win(u, s, 0.0088, Open::Casement));
                }
                back.push(win(u, s, if b % 2 == 0 { 0.0088 } else { 0.0056 }, Open::Casement));
            }
        }
    }
    let ends: Vec<Opening> = if k.far() { Vec::new() } else { (0..n).map(|s| win(dep * 0.5, s, 0.0056, Open::Casement)).collect() };
    k.walls(&rg, y0, h, &wall, [Side::with(&front), Side::with(&ends), Side::with(&back), Side::with(&ends)], true, true);
    let roof = if r(3) < 0.6 { *pal.roofs.tile_dark } else { *pal.slate.house };
    let rise = dep * 0.5 * 0.62;
    k.roof(&rg, &RoofSpec { h, rise, eaves: 0.0034, verge: 0.0, ends: [End::Hip, End::Hip], col: roof, gable: brick, thick: 0.0012 });
    // stacks along the ridge: the flats had their fires
    let ridge = len * 0.5 - rise;
    let stacks = ((ridge * 2.0) / 0.05).floor().max(1.0) as usize;
    for i in 0..=stacks {
        let x = -ridge + 2.0 * ridge * i as f32 / stacks.max(1) as f32;
        k.stack(x, 0.0, 0.008, 0.0056, h + rise * 0.5, h + rise + 0.005, *pal.brick.chimney, 3);
    }
    if k.far() {
        return;
    }
    // balconies on the front between the stairs, from the first floor: a slab of concrete and a painted front
    let rail = if r(4) < 0.5 { t.accents[1] } else { *pal.walls.window_frame };
    for b in 0..f.bays {
        if stair(b) || b % 4 == 3 {
            continue;
        }
        let x = -len * 0.5 + bay * (b as f32 + 0.5);
        for s in 1..n {
            let y = s as f32 * STOREY;
            k.boxy(x - bay * 0.48, x + bay * 0.48, dep * 0.5, dep * 0.5 + 0.0042, y - thick(&k, 0.0012) * 0.66, y + thick(&k, 0.0012) * 0.34, t.concrete, Some(t.concrete));
            k.boxy(x - bay * 0.48, x + bay * 0.48, dep * 0.5 + 0.0042 - thick(&k, 0.0008), dep * 0.5 + 0.0042, y + 0.0004, y + 0.0058, rail, Some(shade(rail, 1.05)));
        }
    }
    if k.near() {
        // the concrete band at each floor, and the stairs' canopies
        for s in 1..n {
            let y = s as f32 * STOREY;
            band_round(&mut k, &rg, y - 0.0006, y + 0.0004, 0.0005, t.concrete, [true, true, true, true]);
        }
        for b in (0..f.bays).filter(|&b| stair(b)) {
            let x = -len * 0.5 + bay * (b as f32 + 0.5);
            k.boxy(x - 0.0045, x + 0.0045, dep * 0.5, dep * 0.5 + 0.004, FOOT + 0.0142, FOOT + 0.0154, t.concrete, Some(t.concrete));
        }
    }
}

//// The faces of a block built on a frame: on each face a pane of glass to
/// every bay of every storey from the first, let into the wall's panels
/// (`Open::Glazed`, the glass of the 1960s), and the frame's columns
/// standing proud of it every `col_every` bays, clear to the top.
#[allow(clippy::too_many_arguments)]
fn framed_faces(k: &mut Kit, rg: &Range, y0: f32, storeys: usize, faces: [bool; 4], wall: &Wall, glass: Rgba, col_every: f32, extra: [&[Opening]; 4]) {
    let h = storeys as f32 * STOREY;
    let pane = Open::Glazed { glass, cols: 2, rows: 1 };
    let ops: Vec<Vec<Opening>> = (0..4)
        .map(|i| {
            let mut v: Vec<Opening> = if k.far() || !faces[i] { Vec::new() } else { rows(rg.face(i).len, 1, storeys, BAY - 0.0026, 0.0098, 0.0052, pane, BAY) };
            v.extend_from_slice(extra[i]);
            v
        })
        .collect();
    k.walls(rg, y0, h, wall, [Side::free(&ops[0]), Side::free(&ops[1]), Side::free(&ops[2]), Side::free(&ops[3])], false, false);
    if k.far() {
        return;
    }
    // the frame's columns, full height
    for (i, on) in faces.iter().enumerate() {
        if !*on {
            continue;
        }
        let len = rg.face(i).len;
        let cols = (len / (BAY * col_every)).round().max(1.0) as usize;
        let w = thick(k, 0.0014);
        for c in 0..=cols {
            let u = (len * c as f32 / cols as f32).clamp(w * 0.5, len - w * 0.5);
            k.strip(rg, i, u - w * 0.5, u + w * 0.5, y0.max(STOREY), h - 0.0006, 0.0011, wall.sill);
        }
    }
}

/// A slab block: eight to thirteen storeys on a concrete frame, a long
/// thin block on its lawn, balconies in the colours of its year on the
/// front, the stairs and lifts in its ends, the lift's motor room on the roof.
#[derive(Clone, Copy, Debug)]
pub struct Slab {
    pub bays: usize,
    pub storeys: u8,
    pub detail: Detail,
    pub seed: Seed,
}

pub const SLAB_DEPTH: f32 = 3.0 * BAY;

impl Slab {
    pub fn length(&self) -> f32 {
        self.bays as f32 * BAY
    }
}

/// The ground floor of a block of flats on a frame: the entrance, glazed,
/// in the middle of its front, and windows of the caretaker's rooms, the
/// stores and the laundry round the rest.
fn ground_floor(len: f32, dep: f32, entrance: f32, paint: Rgba) -> [Vec<Opening>; 4] {
    let mut front = vec![Opening { u: len * 0.5, y: FOOT, w: entrance, h: 0.0136, kind: Open::Shop { paint, awning: None } }];
    front.extend(rows(len, 0, 1, 0.0082, 0.0082, 0.0064, Open::Casement, BAY * 1.5).into_iter().filter(|o| (o.u - len * 0.5).abs() > entrance * 0.5 + 0.006));
    let side = rows(dep, 0, 1, 0.0062, 0.0082, 0.0064, Open::Casement, BAY * 1.5);
    let mut back = rows(len, 0, 1, 0.0082, 0.0082, 0.0064, Open::Casement, BAY * 1.5);
    if let Some(o) = back.get_mut(1) {
        o.kind = Open::Door { paint, over: crate::kit::Over::Nothing };
        o.y = FOOT;
        o.h = 0.0126;
        o.w = 0.0056;
    }
    [front, side.clone(), back, side]
}

pub(crate) fn slab(solid: &mut MeshBuf, site: &Site, b: &Slab, pal: &Palette) {
    let t = Tones::of(pal);
    let r = |n: i32| b.seed.pick(n);
    let mut k = Kit::new(solid, site, pal, b.detail, 0.0);
    let (len, dep) = (b.length(), SLAB_DEPTH);
    let n = b.storeys as usize;
    let h = n as f32 * STOREY;
    let rg = Range::rect(-len * 0.5, len * 0.5, -dep * 0.5, dep * 0.5);
    let y0 = -k.drop(&rg);
    let concrete = shade(t.concrete, 0.96 + 0.08 * r(1));
    // the panels between the windows are the spandrels'; the frame and the sills are concrete
    let wall = Wall { col: t.spandrel, plinth: Some(t.concrete_dark), frame: *pal.modern.mullion, sill: concrete };
    let ground = Wall { col: t.concrete_dark, plinth: None, frame: *pal.walls.window_frame, sill: concrete };
    let g = if k.far() { Default::default() } else { ground_floor(len, dep, 0.016, t.concrete_dark) };
    k.walls(&rg, y0, STOREY, &ground, [Side::with(&g[0]), Side::with(&g[1]), Side::with(&g[2]), Side::with(&g[3])], true, false);
    // the ends are blank but for a slit of light up the stair
    let stair: Vec<Opening> = if k.far() { Vec::new() } else { (1..n).map(|s| Opening { u: dep * 0.5, y: s as f32 * STOREY + 0.004, w: 0.0036, h: 0.0145, kind: Open::Paned { cols: 1, rows: 2, round: false, lit: false } }).collect() };
    framed_faces(&mut k, &rg, STOREY, n, [true, false, true, false], &Wall { col: concrete, ..wall }, t.glass_office, 2.0, [&[], &stair, &[], &stair]);
    flat_roof(&mut k, -len * 0.5, len * 0.5, -dep * 0.5, dep * 0.5, h, 0.0028, concrete, t.felt);
    // the lift's motor room and the tank, a third along
    let mx = -len * 0.5 + len * (0.28 + 0.1 * r(2));
    k.boxy(mx - 0.009, mx + 0.009, -0.009, 0.009, h, h + 0.013, concrete, Some(t.felt));
    if k.far() {
        return;
    }
    // balconies in the colours of the 1960s, every other pair of bays, from the second floor
    let colour = t.accents[(r(3) * 4.0) as usize % 4];
    let pairs = b.bays / 2;
    let (slab_t, rail_t) = (thick(&k, 0.001), thick(&k, 0.0008));
    for p in 0..pairs {
        if (p + (r(4) * 2.0) as usize) % 2 == 1 {
            continue;
        }
        let x0 = -len * 0.5 + p as f32 * 2.0 * BAY + 0.0016;
        let x1 = x0 + 2.0 * BAY - 0.0032;
        for s in 1..n {
            let y = s as f32 * STOREY;
            k.boxy(x0, x1, dep * 0.5, dep * 0.5 + 0.0046, y - slab_t * 0.6, y + slab_t * 0.4, concrete, Some(concrete));
            k.boxy(x0, x1, dep * 0.5 + 0.0046 - rail_t, dep * 0.5 + 0.0046, y + 0.0004, y + 0.0062, colour, Some(shade(colour, 1.05)));
        }
    }
    // the canopy over the door
    k.boxy(-0.012, 0.012, dep * 0.5, dep * 0.5 + 0.007, STOREY - 0.002, STOREY - 0.0004, concrete, Some(concrete));
}

/// A point block: ten to twenty-two storeys on a square of five bays, a
/// pane of glass to every bay of every storey, the balconies at its
/// corners, a stair and lift core up one side, its plant on the roof.
#[derive(Clone, Copy, Debug)]
pub struct PointBlock {
    pub storeys: u8,
    pub detail: Detail,
    pub seed: Seed,
}

pub const POINT_SIDE: f32 = 5.0 * BAY;

pub(crate) fn point_block(solid: &mut MeshBuf, site: &Site, b: &PointBlock, pal: &Palette) {
    let t = Tones::of(pal);
    let r = |n: i32| b.seed.pick(n);
    let mut k = Kit::new(solid, site, pal, b.detail, 0.0);
    let s = POINT_SIDE * 0.5;
    let n = b.storeys as usize;
    let h = n as f32 * STOREY;
    let rg = Range::rect(-s, s, -s, s);
    let y0 = -k.drop(&rg);
    let concrete = shade(t.concrete, 0.95 + 0.1 * r(1));
    let wall = Wall { col: concrete, plinth: Some(t.concrete_dark), frame: *pal.modern.mullion, sill: concrete };
    let ground = Wall { col: t.concrete_dark, frame: *pal.walls.window_frame, ..wall };
    let g = if k.far() { Default::default() } else { ground_floor(POINT_SIDE, POINT_SIDE, 0.014, t.concrete_dark) };
    k.walls(&rg, y0, STOREY, &ground, [Side::with(&g[0]), Side::with(&g[1]), Side::with(&g[2]), Side::with(&g[3])], true, false);
    // the west face: panes either side of the core
    let west: Vec<Opening> = if k.far() { Vec::new() } else { rows(POINT_SIDE, 1, n, BAY - 0.0026, 0.0098, 0.0052, Open::Glazed { glass: shade(t.glass_office, 0.95), cols: 2, rows: 1 }, BAY).into_iter().filter(|o| (o.u - s).abs() > 0.016).collect() };
    framed_faces(&mut k, &rg, STOREY, n, [true, true, true, false], &wall, t.glass_office, 2.5, [&[], &[], &[], &west]);
    // the core up the left side: a tower of plain concrete standing a little proud, with a slit of light
    let core = Range::rect(-s - 0.004, -s + 0.0002, -0.012, 0.012);
    let slit: Vec<Opening> = if k.far() { Vec::new() } else { (1..n).map(|st| Opening { u: 0.012, y: st as f32 * STOREY + 0.004, w: 0.0036, h: 0.0145, kind: Open::Paned { cols: 1, rows: 2, round: false, lit: false } }).collect() };
    k.walls(&core, y0, h + 0.012, &Wall { col: shade(concrete, 0.92), ..wall }, [Side::PLAIN, Side::NONE, Side::PLAIN, Side::free(&slit)], true, false);
    flat_roof(&mut k, -s - 0.004, -s + 0.0002, -0.012, 0.012, h + 0.012, 0.0, concrete, t.felt);
    flat_roof(&mut k, -s, s, -s, s, h, 0.0028, concrete, t.felt);
    k.boxy(-0.01, 0.014, -0.014, 0.008, h, h + 0.011, concrete, Some(t.felt));
    if k.far() {
        return;
    }
    // balconies at the two front corners, every floor, in the block's colour
    let colour = t.accents[(r(3) * 4.0) as usize % 4];
    let (slab_t, rail_t) = (thick(&k, 0.001), thick(&k, 0.0008));
    for st in 1..n {
        let y = st as f32 * STOREY;
        for side in [-1.0f32, 1.0] {
            let (x0, x1) = if side > 0.0 { (s - 0.015, s + 0.003) } else { (-s + 0.002, -s + 0.017) };
            k.boxy(x0, x1, s, s + 0.0045, y - slab_t * 0.6, y + slab_t * 0.4, concrete, Some(concrete));
            k.boxy(x0, x1, s + 0.0045 - rail_t, s + 0.0045, y + 0.0004, y + 0.006, colour, Some(shade(colour, 1.05)));
        }
    }
    k.boxy(-0.011, 0.011, s, s + 0.007, STOREY - 0.002, STOREY - 0.0004, concrete, Some(concrete));
}
