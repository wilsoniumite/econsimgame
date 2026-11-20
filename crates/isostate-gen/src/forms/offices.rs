//! Offices, from the "grown" direction: what the dearest ground of a town
//! is rebuilt as, age by age. A bank of stone with chambers over it (to
//! 1939), an office of glass on a podium of shops (1955 to 1989), and the
//! tower (from 1990) where the rent is the highest on the island.

use glam::Vec3;

use crate::colour::{pick, shade};
use super::{flat_roof, rows, win, Tones, THIN};
use crate::hash::Seed;
use crate::houses::Detail;
use crate::kit::{End, Kit, Open, Opening, Range, RoofSpec, Side, Wall, FOOT, STOREY};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::scale::BAY;
use crate::site::Site;
use crate::trees::{hedge_tree, Greenery};

/// A bank: a banking hall of a storey and a half under two storeys of
/// offices, in stone, with a portico of columns across its middle bays.
#[derive(Clone, Copy, Debug)]
pub struct Bank {
    pub detail: Detail,
    pub seed: Seed,
}

pub const BANK: (f32, f32) = (6.0 * BAY, 4.0 * BAY);

pub(crate) fn bank(solid: &mut MeshBuf, site: &Site, b: &Bank, pal: &Palette) {
    let r = |n: i32| b.seed.pick(n);
    let mut k = Kit::new(solid, site, pal, b.detail, 0.0);
    let (len, dep) = BANK;
    let (hl, hd) = (len * 0.5, dep * 0.5);
    let hall = 1.5 * STOREY;
    let h = hall + 2.0 * STOREY;
    let rg = Range::rect(-hl, hl, -hd, hd);
    let y0 = -k.drop(&rg);
    let stone = if r(1) < 0.5 { *pal.stone.church } else { *pal.stone.limestone };
    let wall = Wall { col: stone, plinth: Some(shade(stone, 0.8)), frame: *pal.walls.window_frame, sill: shade(stone, 1.04) };
    let mut front = Vec::new();
    let mut side = Vec::new();
    if !k.far() {
        for i in 0..6 {
            let u = len * (i as f32 + 0.5) / 6.0;
            if i == 2 || i == 3 {
                if i == 2 {
                    front.push(Opening { u: len * 0.5, y: FOOT, w: 0.0112, h: 0.0218, kind: Open::Leaves { paint: pick(&pal.walls.door_paints, 0.1), round: true } });
                }
            } else {
                front.push(Opening { u, y: FOOT + 0.006, w: 0.0072, h: 0.019, kind: Open::Paned { cols: 2, rows: 4, round: true, lit: false } });
            }
            for s in 0..2 {
                front.push(Opening { u, y: hall + s as f32 * STOREY + 0.0062, w: 0.0062, h: 0.0084, kind: Open::Sash });
            }
        }
        for i in 0..4 {
            let u = dep * (i as f32 + 0.5) / 4.0;
            side.push(Opening { u, y: FOOT + 0.006, w: 0.0072, h: 0.019, kind: Open::Paned { cols: 2, rows: 4, round: true, lit: false } });
            for s in 0..2 {
                side.push(Opening { u, y: hall + s as f32 * STOREY + 0.0062, w: 0.0062, h: 0.0084, kind: Open::Sash });
            }
        }
    }
    k.walls(&rg, y0, h, &wall, [Side::free(&front), Side::free(&side), Side::free(&side), Side::free(&side)], true, false);
    // a cornice, a balustrade, and a roof of slate behind it
    let lip = 0.0024;
    k.boxy(-hl - lip, hl + lip, -hd - lip, hd + lip, h - 0.0012, h + 0.0016, shade(stone, 1.05), Some(shade(stone, 1.08)));
    flat_roof(&mut k, -hl, hl, -hd, hd, h + 0.0016, 0.0052, stone, *pal.slate.town);
    let inner = Range::rect(-hl + 0.008, hl - 0.008, -hd + 0.008, hd - 0.008);
    k.roof(&inner, &RoofSpec { h: h + 0.0016, rise: 0.011, eaves: 0.0, verge: 0.0, ends: [End::Hip, End::Hip], col: *pal.slate.town, gable: stone, thick: 0.001 });
    if k.far() {
        return;
    }
    // the portico: four columns through the hall and the floor above, an entablature and a pediment
    let top = hall + STOREY;
    let (x0, x1) = (-len / 6.0 - 0.004, len / 6.0 + 0.004);
    let out = 0.0068;
    for c in 0..4 {
        let x = x0 + (x1 - x0) * c as f32 / 3.0;
        let base = k.pt(x, FOOT, hd + out * 0.6);
        let cr = if k.near() { 0.0019 } else { 0.0021 };
        k.solid.frustum_turned(base, cr, cr * 0.86, top - FOOT, if k.near() { 8 } else { 5 }, 0.0, 0.0, shade(stone, 1.06), None);
    }
    k.boxy(x0 - 0.003, x1 + 0.003, hd, hd + out + 0.0014, top, top + 0.0042, shade(stone, 1.04), Some(shade(stone, 1.1)));
    let ped = Range::rect_across(x0 - 0.003, x1 + 0.003, hd, hd + out + 0.0014);
    k.roof(&ped, &RoofSpec { h: top + 0.0042, rise: 0.0085, eaves: 0.0012, verge: 0.0012, ends: [End::Party, End::Gable], col: *pal.slate.town, gable: shade(stone, 1.02), thick: 0.0012 });
    // steps up to it
    k.boxy(x0 - 0.002, x1 + 0.002, hd, hd + out + 0.003, 0.0, FOOT, shade(stone, 0.95), Some(shade(stone, 1.06)));
    if k.near() {
        // the rustication of the hall: a band at each course
        let mut y = FOOT + 0.003;
        while y < hall - 0.001 {
            for i in 0..4 {
                let l = rg.face(i).len;
                k.strip(&rg, i, 0.0, l, y, y + 0.0004, 0.0003, shade(stone, 0.86));
            }
            y += 0.0036;
        }
        for i in 0..4 {
            let l = rg.face(i).len;
            k.strip(&rg, i, 0.0, l, hall - 0.0005, hall + 0.0009, 0.0008, shade(stone, 1.06));
        }
    }
}

//// An office of the 1960s and 70s: a podium of two storeys of shops and
/// offices filling its block, and over its back half a slab of a curtain
/// wall, a pane of grey glass to every bay of every floor over a spandrel,
/// its ends blank.
#[derive(Clone, Copy, Debug)]
pub struct Office {
    /// storeys of the slab above the podium: from the rent its ground fetches
    pub storeys: u8,
    pub detail: Detail,
    pub seed: Seed,
}

pub const OFFICE: (f32, f32) = (10.0 * BAY, 6.0 * BAY);

pub(crate) fn office(solid: &mut MeshBuf, site: &Site, o: &Office, pal: &Palette) {
    let t = Tones::of(pal);
    let r = |n: i32| o.seed.pick(n);
    let mut k = Kit::new(solid, site, pal, o.detail, 0.0);
    let (len, dep) = OFFICE;
    let (hl, hd) = (len * 0.5, dep * 0.5);
    let podium = Range::rect(-hl, hl, -hd, hd);
    let y0 = -k.drop(&podium);
    let ph = 2.0 * STOREY;
    let clad = if r(1) < 0.5 { t.spandrel } else { t.concrete };
    let wall = Wall { col: clad, plinth: Some(t.concrete_dark), frame: *pal.modern.mullion, sill: clad };
    let ribbon = Open::Glazed { glass: t.glass_office, cols: 3, rows: 1 };
    let mut front = Vec::new();
    let mut ends = Vec::new();
    let mut back = Vec::new();
    if !k.far() {
        for i in 0..5 {
            let u = len * (i as f32 + 0.5) / 5.0;
            let awning = if r(10 + i) < 0.5 { Some(pick(&pal.roofs.awnings, r(20 + i))) } else { None };
            front.push(Opening { u, y: FOOT, w: 2.0 * BAY - 0.004, h: 0.0132, kind: Open::Shop { paint: pick(&pal.walls.door_paints, r(30 + i)), awning } });
        }
        for i in 0..3 {
            ends.push(win(dep * (i as f32 + 0.5) / 3.0, 0, 0.0088, Open::Casement));
        }
        // the first floor: a ribbon of glass round the podium, a pane to every two bays
        front.extend(rows(len, 1, 2, 2.0 * BAY - 0.003, 0.0096, 0.0055, ribbon, 2.0 * BAY));
        ends.extend(rows(dep, 1, 2, 2.0 * BAY - 0.003, 0.0096, 0.0055, ribbon, 2.0 * BAY));
        back = rows(len, 0, 2, 2.0 * BAY - 0.003, 0.0096, 0.0055, ribbon, 2.0 * BAY);
        // the goods' door at the back
        back[1] = Opening { u: back[1].u, y: 0.0, w: 0.016, h: 0.0146, kind: Open::Leaves { paint: t.concrete_dark, round: false } };
    }
    k.walls(&podium, y0, ph, &wall, [Side::free(&front), Side::free(&ends), Side::free(&back), Side::free(&ends)], true, false);
    flat_roof(&mut k, -hl, hl, -hd, hd, ph, 0.0024, clad, t.felt);
    // the slab over the back of the podium
    let n = o.storeys as usize;
    let (sz0, sz1) = (-hd + 0.004, -hd + 0.004 + 3.0 * BAY);
    let (sx0, sx1) = (-hl + 0.006, hl - 0.006);
    let slab = Range::rect(sx0, sx1, sz0, sz1);
    let top = ph + n as f32 * STOREY;
    let curtain = Wall { col: t.spandrel, plinth: None, frame: *pal.modern.mullion, sill: t.spandrel };
    let panes: Vec<Opening> = if k.far() { Vec::new() } else { rows(slab.len, 2, n + 2, BAY - 0.0022, 0.0118, 0.006, Open::Glazed { glass: t.glass_office, cols: 2, rows: 1 }, BAY) };
    k.walls(&slab, ph, top, &curtain, [Side::free(&panes), Side::NONE, Side::free(&panes), Side::NONE], false, false);
    // the blank ends, in the podium's cladding, standing a little proud
    for (x0, x1) in [(sx0 - 0.002, sx0 + 0.0004), (sx1 - 0.0004, sx1 + 0.002)] {
        k.boxy(x0, x1, sz0 - 0.001, sz1 + 0.001, ph, top + 0.003, clad, Some(shade(clad, 1.05)));
    }
    flat_roof(&mut k, sx0, sx1, sz0, sz1, top, 0.0, clad, t.felt);
    k.boxy(sx0 + 0.02, sx0 + 0.045, sz0 + 0.008, sz1 - 0.008, top, top + 0.011, clad, Some(t.felt));
}

/// A tower: where the rent is highest on the island, from 1990. The tower
/// of the 1990s is clad in stone-coloured panels with a pane of dark glass
/// to every bay, under a pyramid of copper; the tower of the new century is
/// light glass storey-high in a white frame with fins, a garden on its
/// setback and a lit crown. Its storeys are the rent's.
#[derive(Clone, Copy, Debug)]
pub struct Tower {
    pub storeys: u8,
    /// the year it went up: the style follows
    pub built: i32,
    pub detail: Detail,
    pub seed: Seed,
}

pub const TOWER: (f32, f32) = (7.0 * BAY, 6.0 * BAY);

pub(crate) fn tower(solid: &mut MeshBuf, green: &mut Greenery, site: &Site, tw: &Tower, pal: &Palette) {
    let t = Tones::of(pal);
    let mut k = Kit::new(solid, site, pal, tw.detail, 0.0);
    let (len, dep) = TOWER;
    let (hl, hd) = (len * 0.5, dep * 0.5);
    let n = tw.storeys as usize;
    let late = tw.built >= 2010;
    let base = Range::rect(-hl - 0.006, hl + 0.006, -hd - 0.006, hd + 0.006);
    let y0 = -k.drop(&base);
    // the base: two storeys of stone and glass, wider than the shaft
    let stone = if late { t.cladding_white } else { *pal.modern.stone_panel };
    let lobby = Wall { col: stone, plinth: Some(shade(stone, 0.85)), frame: *pal.modern.mullion, sill: stone };
    let hall = Open::Glazed { glass: if late { t.glass_tower } else { t.glass_office }, cols: 4, rows: 3 };
    let mut doors = Vec::new();
    let mut back = Vec::new();
    if !k.far() {
        doors.push(Opening { u: base.len * 0.5, y: FOOT, w: 0.022, h: 0.026, kind: hall });
        for u in [0.016, base.len - 0.016] {
            doors.push(Opening { u, y: FOOT, w: 0.014, h: 0.026, kind: Open::Glazed { glass: if late { t.glass_tower } else { t.glass_office }, cols: 2, rows: 3 } });
        }
        back = rows(base.len, 0, 2, 0.009, 0.0096, 0.0056, Open::Glazed { glass: t.glass_office, cols: 2, rows: 1 }, 2.0 * BAY);
    }
    k.walls(&base, y0, 2.0 * STOREY, &lobby, [Side::free(&doors), Side::free(&doors[..doors.len().min(1)]), Side::free(&back), Side::free(&doors[..doors.len().min(1)])], true, false);
    flat_roof(&mut k, base.cx - base.len * 0.5, base.cx + base.len * 0.5, -hd - 0.006, hd + 0.006, 2.0 * STOREY, 0.003, stone, t.felt);
    // the shaft, set back once two thirds of the way up
    let set = (n as f32 * (0.62 + 0.12 * tw.seed.pick(1))).round() as usize;
    let shafts = [(2usize, set, hl, hd), (set, n, hl - BAY, hd - BAY * 0.5)];
    let (glass, frame) = if late { (t.glass_tower, t.cladding_white) } else { (shade(t.glass_office, 0.8), stone) };
    for &(s0, s1, sx, sz) in &shafts {
        let rg = Range::rect(-sx, sx, -sz, sz);
        let (ya, yb) = (s0 as f32 * STOREY, s1 as f32 * STOREY);
        let w = Wall { col: frame, plinth: None, frame: if late { t.cladding_white } else { *pal.modern.mullion }, sill: stone };
        // a pane to every bay: storey-high in the new century, under a band of stone panel before it
        let (pw, ph, sill, kind) = if late { (BAY - 0.0024, STOREY - 0.0034, 0.0017, Open::Glazed { glass, cols: 1, rows: 1 }) } else { (BAY - 0.0026, 0.0098, 0.0074, Open::Glazed { glass, cols: 2, rows: 1 }) };
        let (along, across) = if k.far() { (Vec::new(), Vec::new()) } else { (rows(rg.len, s0, s1, pw, ph, sill, kind, BAY), rows(rg.dep, s0, s1, pw, ph, sill, kind, BAY)) };
        k.walls(&rg, ya, yb, &w, [Side::free(&along), Side::free(&across), Side::free(&along), Side::free(&across)], false, false);
        flat_roof(&mut k, -sx, sx, -sz, sz, yb, 0.0026, stone, t.felt);
        if k.far() {
            continue;
        }
        for i in 0..4 {
            let l = rg.face(i).len;
            if late {
                // white fins a bay apart, standing out from the glass
                let m = (l / BAY).floor().max(2.0) as usize;
                for j in 0..=m {
                    let u = (l * j as f32 / m as f32).clamp(0.0008, l - 0.0008);
                    k.strip(&rg, i, u - 0.0008, u + 0.0008, ya, yb, 0.0014, t.cladding_white);
                }
            } else {
                // piers of the stone every two bays
                let m = (l / (2.0 * BAY)).round().max(1.0) as usize;
                for j in 0..=m {
                    let u = (l * j as f32 / m as f32).clamp(0.0009, l - 0.0009);
                    k.strip(&rg, i, u - 0.0009, u + 0.0009, ya, yb, 0.0009, shade(stone, 0.94));
                }
            }
        }
    }
    let (sx, sz) = (hl - BAY, hd - BAY * 0.5);
    let top = n as f32 * STOREY;
    let mast = if k.near() { 0.0012 } else { THIN };
    if late {
        // a garden on the setback, and the crown: an open frame two storeys high, lit warm at its head
        let y = set as f32 * STOREY + 0.0008;
        if !k.far() {
            k.patch(-hl + 0.001, hl - 0.001, sz + 0.0005, hd - 0.001, y - k.ground(0.0, 0.0), *pal.ground.garden);
        }
        for (i, x) in [-hl + 0.006, 0.0, hl - 0.006].iter().enumerate() {
            let p = k.pt(*x, y, (sz + hd) * 0.5);
            hedge_tree(green, &site.moved(p, 0.0), 0.2, i as f32, i as f32 * 2.0, pal);
        }
        let crown = 2.0 * STOREY;
        if k.far() {
            for (x0, x1, z0, z1) in [(-sx, -sx + 0.0016, -sz, sz), (sx - 0.0016, sx, -sz, sz), (-sx, sx, -sz, -sz + 0.0016), (-sx, sx, sz - 0.0016, sz)] {
                k.boxy(x0, x1, z0, z1, top, top + crown, t.cladding_white, Some(t.cladding_white));
            }
        } else {
            // the frame is open: a post at every bay round it, the sky between
            let p = 0.0009;
            for (along, half_other, on_x) in [(sx, sz, true), (sz, sx, false)] {
                let m = ((2.0 * along) / BAY).round().max(2.0) as usize;
                for j in 0..=m {
                    let a = -along + 2.0 * along * j as f32 / m as f32;
                    let a = a.clamp(-along + p, along - p);
                    for side in [-1.0f32, 1.0] {
                        let o = side * (half_other - p);
                        let (x, z) = if on_x { (a, o) } else { (o, a) };
                        k.boxy(x - p, x + p, z - p, z + p, top, top + crown - 0.004, t.cladding_white, None);
                    }
                }
            }
        }
        if !k.far() {
            let band = *pal.walls.ochre;
            for (x0, x1, z0, z1) in [(-sx - 0.0006, -sx + 0.0022, -sz - 0.0006, sz + 0.0006), (sx - 0.0022, sx + 0.0006, -sz - 0.0006, sz + 0.0006), (-sx, sx, -sz - 0.0006, -sz + 0.0022), (-sx, sx, sz - 0.0022, sz + 0.0006)] {
                k.boxy(x0, x1, z0, z1, top + crown - 0.004, top + crown, band, Some(shade(band, 1.05)));
            }
        }
        let m = k.pt(0.0, top, 0.0);
        k.solid.beam(m, m + Vec3::Y * (crown + 0.035), mast, t.cladding_white);
    } else {
        // a pyramid of green copper over a band of stone
        let rg = Range::rect(-sx, sx, -sz, sz);
        let copper = *pal.modern.copper;
        k.roof(&rg, &RoofSpec { h: top, rise: sz * 1.5, eaves: 0.0018, verge: 0.0, ends: [End::Hip, End::Hip], col: copper, gable: stone, thick: 0.0012 });
        let p = k.pt(0.0, top + sz * 1.5, 0.0);
        k.solid.beam(p, p + Vec3::Y * 0.012, mast, stone);
    }
}
