//! The sheds at a town's edge, by the motorway's junctions: the retail
//! shed in its car park, and the distribution shed with its docks and its
//! lorries. They are drawn smaller than they would be at the houses' scale,
//! as a great works is (a shed of 200 m would be half a map cell): a shed
//! grows by bays, and a great one by more of them, slowly.

use glam::Vec3;

use crate::colour::{shade, Rgba};
use super::{flat_roof, Tones};
use crate::hash::Seed;
use crate::houses::Detail;
use crate::kit::{Kit, Open, Opening, Range, Side, Wall, FOOT, STOREY};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::scale::BAY;
use crate::site::Site;
use crate::trees::{hedge_tree, Greenery};

/// A retail shed: a hall a storey and a half high in profiled cladding,
/// its maker's band across its front, a glazed entrance, and its car park
/// before it. `size` is its floor over a standard one's (the shops' trade
/// it stands for).
#[derive(Clone, Copy, Debug)]
pub struct RetailShed {
    pub size: f32,
    pub detail: Detail,
    pub seed: Seed,
}

impl RetailShed {
    pub fn plan(&self) -> (f32, f32) {
        let g = self.size.clamp(0.4, 4.0).sqrt();
        (0.15 * g, 0.1 * g)
    }
    /// how deep its car park is, before it
    pub const PARK: f32 = 0.085;
}

pub(crate) fn retail_shed(solid: &mut MeshBuf, flat: &mut MeshBuf, green: &mut Greenery, site: &Site, s: &RetailShed, pal: &Palette) {
    let t = Tones::of(pal);
    let r = |n: i32| s.seed.pick(n);
    let mut k = Kit::new(solid, site, pal, s.detail, 0.0);
    let (len, dep) = s.plan();
    let (hl, hd) = (len * 0.5, dep * 0.5);
    let h = 1.6 * STOREY;
    let rg = Range::rect(-hl, hl, -hd, hd);
    let y0 = -k.drop(&rg);
    let clad = [t.cladding_blue, t.cladding_green, t.cladding_white][(r(1) * 3.0) as usize % 3];
    let band = t.accents[(r(2) * 4.0) as usize % 4];
    let wall = Wall { col: clad, plinth: Some(t.concrete_dark), frame: t.concrete_dark, sill: t.concrete };
    let mut back = Vec::new();
    if !k.far() {
        back.push(Opening { u: len * 0.25, y: 0.0, w: 0.016, h: 0.018, kind: Open::Leaves { paint: shade(clad, 0.8), round: false } });
    }
    k.walls(&rg, y0, h, &wall, [Side::PLAIN, Side::PLAIN, Side::with(&back), Side::PLAIN], true, false);
    flat_roof(&mut k, -hl, hl, -hd, hd, h, 0.0, clad, t.shed_roof);
    // the maker's band across the top of the front and round its corners
    k.boxy(-hl - 0.0008, hl + 0.0008, hd - 0.004, hd + 0.0008, h - 0.008, h + 0.0035, band, Some(shade(band, 1.06)));
    for x in [-hl - 0.0008, hl - 0.006] {
        k.boxy(x, x + 0.0068, -hd + 0.01, hd - 0.004, h - 0.008, h + 0.0035, band, Some(shade(band, 1.06)));
    }
    // the entrance: a glazed box standing forward, a little higher than the hall
    let ex = -hl * 0.35;
    let (e0, e1) = (ex - 0.016, ex + 0.016);
    k.boxy(e0, e1, hd, hd + 0.01, 0.0, h + 0.006, t.glass_tower, Some(t.cladding_white));
    if !k.far() {
        k.boxy(e0 - 0.002, e1 + 0.002, hd, hd + 0.013, h + 0.002, h + 0.0072, t.cladding_white, Some(t.cladding_white));
        // trolleys' shelter and the windows of the café along the front
        let glass = t.glass_office;
        k.strip(&rg, 0, len * 0.6, len * 0.92, FOOT + 0.002, FOOT + 0.012, 0.0006, glass);
    }
    cladding(&mut k, &rg, h - 0.008, clad, [FOOT; 4]);
    // the car park before it, its bays lined, and the cars in it
    car_park(flat, &mut k, -hl, hl, hd + 0.014, hd + RetailShed::PARK, 0.55 + 0.35 * r(3), s.seed, &t);
    for (i, x) in [-hl, -hl * 0.33, hl * 0.33, hl].iter().enumerate() {
        let p = k.pt(*x, 0.0, hd + RetailShed::PARK + 0.004);
        hedge_tree(green, &site.moved(p, 0.0), 0.24 + 0.06 * r(10 + i as i32), r(20 + i as i32) * 3.0, r(30 + i as i32) * 6.0, pal);
    }
}

/// The skin of a shed of a steel frame: from the play distance the frame's
/// stanchions showing through the cladding a bay apart; close to, the ribs
/// of the profiled sheets between them as well, four to a bay, and the rail
/// where the lower sheets meet the upper.
fn cladding(k: &mut Kit, rg: &Range, top: f32, clad: Rgba, from: [f32; 4]) {
    if k.far() {
        return;
    }
    for i in 0..4 {
        let foot = from[i];
        let l = rg.face(i).len;
        let m = (l / BAY).round().max(1.0) as usize;
        for j in 0..=m {
            let u = (l * j as f32 / m as f32).clamp(0.0008, l - 0.0008);
            k.strip(rg, i, u - 0.0008, u + 0.0008, foot, top, 0.0006, shade(clad, 0.92));
        }
        if k.near() {
            for j in 0..m * 4 {
                if j % 4 == 0 {
                    continue;
                }
                let u = l * j as f32 / (m * 4) as f32;
                k.strip(rg, i, u - 0.0002, u + 0.0002, foot, top, 0.0003, shade(clad, 0.95));
            }
            k.strip(rg, i, 0.0, l, top * 0.42, top * 0.42 + 0.0008, 0.0005, shade(clad, 0.86));
        }
    }
}

/// A car park from x0 to x1 and z0 to z1, its rows of bays across x, `full`
/// of it taken. The tarmac lies in `flat`; the cars are built.
#[allow(clippy::too_many_arguments)]
pub(crate) fn car_park(flat: &mut MeshBuf, k: &mut Kit, x0: f32, x1: f32, z0: f32, z1: f32, full: f32, seed: Seed, t: &Tones) {
    let y = 0.0016;
    let corner = |k: &Kit, x: f32, z: f32| k.pt(x, y, z);
    flat.quad(corner(k, x0, z1), corner(k, x1, z1), corner(k, x1, z0), corner(k, x0, z0), t.tarmac_light);
    let (bay_w, bay_d, aisle) = (0.0058, 0.011, 0.012);
    let mut z = z0 + 0.003;
    let mut row = 0;
    while z + bay_d * 2.0 + aisle < z1 + 0.001 {
        for half in 0..2 {
            let zr = z + half as f32 * (bay_d + aisle);
            let n = ((x1 - x0 - 0.006) / bay_w).floor() as i32;
            for i in 0..n {
                let x = x0 + 0.003 + i as f32 * bay_w;
                if !k.far() {
                    let a = k.pt(x, y + 0.0003, zr);
                    let b = k.pt(x + 0.0005, y + 0.0003, zr + bay_d);
                    flat.quad(Vec3::new(a.x, a.y, b.z), Vec3::new(b.x, a.y, b.z), Vec3::new(b.x, a.y, a.z), a, t.line);
                }
                let h = crate::hash::hash2(i, row * 2 + half, (seed.u * 9973.0) as i32);
                if h < full {
                    let paint = t.accents[((h * 97.0) as usize) % 4];
                    car(k, x + bay_w * 0.5, zr + bay_d * 0.5, true, paint, h);
                }
            }
        }
        z += 2.0 * (bay_d + aisle) * 0.5 + bay_d + aisle * 0.5;
        row += 1;
    }
}

/// A car: a body and a cabin, standing at x, z in the kit's frame, along z
/// (`across`) or along x. Its colour is its owner's.
pub(crate) fn car(k: &mut Kit, x: f32, z: f32, across: bool, paint: Rgba, n: f32) {
    let (l, w) = (0.0084, 0.0042);
    let (hx, hz) = if across { (w * 0.5, l * 0.5) } else { (l * 0.5, w * 0.5) };
    let body = if n > 0.7 { paint } else { super::paintwork(k.pal, n) };
    k.boxy(x - hx, x + hx, z - hz, z + hz, 0.0006, 0.0034, body, Some(shade(body, 1.08)));
    if k.far() {
        return;
    }
    let (cx, cz) = if across { (hx * 0.86, hz * 0.55) } else { (hx * 0.55, hz * 0.86) };
    let glass = *k.pal.glass.window;
    k.boxy(x - cx, x + cx, z - cz, z + cz, 0.0034, 0.0056, shade(glass, 1.2), Some(shade(body, 1.04)));
}

/// A distribution shed: a great hall of two storeys' height, banded in two
/// claddings, lit from its roof, a row of docks along its front with the
/// lorries backed up to them, an office at its corner and its yard before it.
#[derive(Clone, Copy, Debug)]
pub struct DistShed {
    pub size: f32,
    pub detail: Detail,
    pub seed: Seed,
}

impl DistShed {
    pub fn plan(&self) -> (f32, f32) {
        let g = self.size.clamp(0.4, 4.0).sqrt();
        (0.3 * g, 0.17 * g)
    }
    pub const YARD: f32 = 0.07;
}

pub(crate) fn dist_shed(solid: &mut MeshBuf, flat: &mut MeshBuf, site: &Site, s: &DistShed, pal: &Palette) {
    let t = Tones::of(pal);
    let r = |n: i32| s.seed.pick(n);
    let mut k = Kit::new(solid, site, pal, s.detail, 0.0);
    let (len, dep) = s.plan();
    let (hl, hd) = (len * 0.5, dep * 0.5);
    let h = 2.2 * STOREY;
    let rg = Range::rect(-hl, hl, -hd, hd);
    let y0 = -k.drop(&rg);
    let (upper, lower) = if r(1) < 0.5 { (t.cladding_white, t.cladding_blue) } else { (t.cladding_white, t.cladding_green) };
    let wall = Wall { col: upper, plinth: Some(t.concrete_dark), frame: t.concrete_dark, sill: t.concrete };
    // the docks: a door every two bays along most of the front
    let dock_w = 2.0 * BAY;
    let docks = ((len * 0.82) / dock_w).floor() as usize;
    let d0 = -hl + (len - docks as f32 * dock_w) * 0.5 + 0.015;
    let mut front = Vec::new();
    if !k.far() {
        for i in 0..docks {
            let u = d0 + hl + dock_w * (i as f32 + 0.5);
            front.push(Opening { u, y: 0.0028, w: 0.0105, h: 0.0124, kind: Open::Leaves { paint: shade(lower, 0.85), round: false } });
        }
    }
    k.walls(&rg, y0, h, &wall, [Side::free(&front), Side::PLAIN, Side::PLAIN, Side::PLAIN], true, false);
    if !k.far() {
        // the lower band of the second cladding
        for i in 1..4 {
            let l = rg.face(i).len;
            k.strip(&rg, i, 0.0, l, FOOT, h * 0.42, 0.0004, lower);
        }
        k.strip(&rg, 0, 0.0, d0 + hl - 0.001, FOOT, h * 0.42, 0.0004, lower);
        k.strip(&rg, 0, d0 + hl + docks as f32 * dock_w + 0.001, len, FOOT, h * 0.42, 0.0004, lower);
        k.strip(&rg, 0, 0.0, len, h * 0.42 + 0.004, h * 0.42 + 0.006, 0.0004, lower);
    }
    // (above the docks' canopies along the front)
    cladding(&mut k, &rg, h, upper, [0.0178, FOOT, FOOT, FOOT]);
    flat_roof(&mut k, -hl, hl, -hd, hd, h, 0.0016, upper, t.shed_roof);
    if !k.far() {
        // roof lights in rows across it
        let rows = (len / 0.04).floor() as usize;
        for i in 0..rows {
            let x = -hl + len * (i as f32 + 0.5) / rows as f32;
            let p = [k.pt(x - 0.003, h + 0.0009, hd - 0.015), k.pt(x + 0.003, h + 0.0009, hd - 0.015), k.pt(x + 0.003, h + 0.0009, -hd + 0.015), k.pt(x - 0.003, h + 0.0009, -hd + 0.015)];
            k.quad(p, *pal.glass.roof_light, Vec3::Y);
        }
    }
    // canopies over the docks, and the lorries at some of them
    for i in 0..docks {
        let x = d0 + dock_w * (i as f32 + 0.5);
        if !k.far() {
            k.boxy(x - 0.0062, x + 0.0062, hd, hd + 0.004, 0.0162, 0.0162 + super::thick(&k, 0.001), lower, Some(shade(lower, 1.05)));
        }
        if crate::hash::hash2(i as i32, 7, (s.seed.v * 7919.0) as i32) < 0.55 {
            lorry(&mut k, x, hd + 0.003, &t, r(40 + i as i32));
        }
    }
    // the office at the front corner: two storeys of glass
    let (ox0, ox1) = (hl - 0.05, hl + 0.004);
    k.boxy(ox0, ox1, hd - 0.002, hd + 0.014, 0.0, 2.0 * STOREY, t.glass_office, Some(t.felt));
    if !k.far() {
        k.boxy(ox0 - 0.0008, ox1 + 0.0008, hd - 0.002, hd + 0.0148, STOREY - 0.001, STOREY + 0.001, t.cladding_white, Some(t.cladding_white));
    }
    // the yard before it, and the cars of those who work there by the office
    let y = 0.0016;
    let c = |k: &Kit, x: f32, z: f32| k.pt(x, y, z);
    flat.quad(c(&k, -hl, hd + DistShed::YARD), c(&k, hl, hd + DistShed::YARD), c(&k, hl, hd), c(&k, -hl, hd), t.tarmac);
    car_park(flat, &mut k, ox0 - 0.02, ox1, hd + DistShed::YARD - 0.03, hd + DistShed::YARD, 0.7, s.seed, &t);
}

/// A lorry backed up to a dock at x, its back at z: a white trailer and a coloured cab.
pub(crate) fn lorry(k: &mut Kit, x: f32, z: f32, t: &Tones, n: f32) {
    let (w, l, hgt) = (0.0062, 0.032, 0.011);
    let trailer = t.cladding_white;
    k.boxy(x - w * 0.5, x + w * 0.5, z, z + l, 0.0022, 0.0022 + hgt, trailer, Some(shade(trailer, 1.04)));
    let cab = t.accents[((n * 31.0) as usize) % 4];
    k.boxy(x - w * 0.5, x + w * 0.5, z + l + 0.0008, z + l + 0.0078, 0.0012, 0.0098, cab, Some(shade(cab, 1.06)));
    if !k.far() {
        let glass = *k.pal.glass.window;
        super::pane_z(k, x - w * 0.42, x + w * 0.42, z + l + 0.0079, 0.006, 0.0092, glass, 1.0);
    }
}

/// A car park of decks on columns, where ground is too dear to park on (from
/// the "tall" direction). Each deck a slab and its parapet, the columns under
/// it, the cars on it in two rows, and the ramp's tower at one end.
#[derive(Clone, Copy, Debug)]
pub struct Deck {
    pub w: f32,
    pub d: f32,
    /// decks above the ground
    pub decks: u8,
    /// how full it is, 0 to 1
    pub full: f32,
    pub detail: Detail,
    pub seed: Seed,
}

pub(crate) fn deck(m: &mut crate::site::Model, site: &Site, p: &Deck, pal: &Palette) {
    let concrete = *pal.modern.concrete;
    let (asphalt, top) = (*pal.ground.tarmac, *pal.modern.gravel);
    let mut k = Kit::new(&mut m.solid, site, pal, p.detail, 0.0);
    let (hw, hd) = (p.w * 0.5 - 0.003, p.d * 0.5 - 0.003);
    let rise = STOREY * 0.8;
    let slab = 0.0024;
    for i in 0..=p.decks as i32 {
        let y = i as f32 * rise;
        if i > 0 {
            // the slab and its parapet
            k.boxy(-hw, hw, -hd, hd, y - slab, y, concrete, Some(asphalt));
            if !k.far() {
                let pt = super::thick(&k, 0.0012);
                for (x0, x1, z0, z1) in [(-hw, hw, hd - pt, hd), (-hw, hw, -hd, -hd + pt), (-hw, -hw + pt, -hd, hd), (hw - pt, hw, -hd, hd)] {
                    k.boxy(x0, x1, z0, z1, y, y + 0.0045, shade(concrete, 1.05), Some(shade(concrete, 1.1)));
                }
            }
        }
        // its columns
        if i < p.decks as i32 && !k.far() {
            let mut x = -hw + 0.004;
            while x < hw {
                for z in [-hd + 0.004, hd - 0.004] {
                    k.boxy(x - 0.0014, x + 0.0014, z - 0.0014, z + 0.0014, y, y + rise - slab, shade(concrete, 0.85), None);
                }
                x += 0.032;
            }
        }
        // the cars on it, in two rows
        if !k.far() {
            let mut n = 0;
            let mut x = -hw + 0.008;
            while x < hw - 0.006 {
                for (row, z) in [(0, -hd * 0.5), (1, hd * 0.5)] {
                    n += 1;
                    if p.seed.pick(n * 7 + i * 131 + row) < p.full {
                        let col = super::paintwork(pal, p.seed.pick(n * 11 + i));
                        let (at, way) = (k.pt(x, y + 0.0008, z), k.dir(0.0, 0.0, 1.0));
                        super::car(k.solid, at, way, col, pal);
                    }
                }
                x += 0.0075;
            }
        }
    }
    // the ramp's tower at one end
    let h = p.decks as f32 * rise;
    k.boxy(hw - 0.012, hw + 0.002, -hd - 0.002, -hd + 0.012, -0.003, h + 0.008, shade(concrete, 1.05), Some(top));
}
