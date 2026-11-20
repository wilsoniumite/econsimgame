//! The detached house of the motor age, in its garden with its drive and
//! its car, from the "tall" direction: the carpet beyond the motor roads.
//! Built with the kit, to the houses' storey, bay, openings and eaves, as
//! the villa is.

use glam::Vec3;

use super::{car, paintwork};
use crate::colour::{pick, shade, Rgba};
use crate::hash::Seed;
use crate::kit::{Detail, End, Kit, Open, Opening, Over, Range, RoofSpec, Side, Wall, FOOT, STOREY};
use crate::palette::Palette;
use crate::site::{Model, Site};
use crate::trees::hedge_tree;

/// The plot a detached house of the motor age takes: wider than a
/// cottage's, for the drive beside the house.
pub const SUBURB_PLOT: (f32, f32) = (0.064, 0.05);

#[derive(Clone, Copy, Debug)]
pub struct Detached {
    pub plot_w: f32,
    pub plot_d: f32,
    /// the year the plot was first built: its square joined the town after the car
    pub built: i32,
    /// cars on the drive: the households' vehicles a head, times the people a house holds
    pub cars: u8,
    pub detail: Detail,
    pub seed: Seed,
}

fn window(u: f32, s: usize, w: f32) -> Opening {
    Opening { u, y: s as f32 * STOREY + 0.0064, w, h: 0.0064, kind: Open::Casement }
}

fn door(u: f32, paint: Rgba) -> Opening {
    Opening { u, y: FOOT, w: 0.0052, h: 0.0126, kind: Open::Door { paint, over: Over::Nothing } }
}

/// A detached house of the motor age. To 1945 a house of two storeys under
/// a hipped roof of tile, with a bay to the front; from 1946 to 1975 a
/// bungalow, long and low under broad eaves, its garage at one end; from
/// 1976 a house of two storeys with a gabled wing to the road and its
/// garage beside. A drive, a lawn, a tree, and the car.
pub(crate) fn detached(m: &mut Model, site: &Site, t: &Detached, pal: &Palette) {
    let (lawn, drive) = (*pal.ground.garden, *pal.ground.tarmac_worn);
    let r = |n: i32| t.seed.pick(n);
    let (pw, pd) = (t.plot_w, t.plot_d);
    let paint = pick(&pal.walls.door_paints, r(4));
    let tile = [*pal.roofs.tile_red, *pal.roofs.tile_dark, *pal.roofs.tile_orange, *pal.slate.house][(r(5) * 4.0) as usize % 4];
    let walls = if r(1) < 0.45 { pal.brick.terrace(r(2)) } else { pal.walls.town_house(r(2)) };
    let wall = Wall { col: walls, plinth: Some(*pal.stone.plinth), frame: *pal.walls.window_frame, sill: *pal.stone.sill };
    // the drive runs up one side of the plot, the house beside it
    let left = r(3) < 0.5;
    let drive_w = 0.014;
    let (hx0, hx1) = if left { (-pw * 0.5 + drive_w + 0.004, pw * 0.5 - 0.004) } else { (-pw * 0.5 + 0.004, pw * 0.5 - drive_w - 0.004) };
    let zf = pd * 0.5 - 0.016;
    let (cx, w) = ((hx0 + hx1) * 0.5, hx1 - hx0);
    let dx = if left { -pw * 0.5 + drive_w * 0.5 + 0.002 } else { pw * 0.5 - drive_w * 0.5 - 0.002 };
    let mut k = Kit::new(&mut m.solid, site, pal, t.detail, 0.0);
    // the lawn and the drive, at every level but far
    if !k.far() {
        k.patch(-pw * 0.5 + 0.0006, pw * 0.5 - 0.0006, -pd * 0.5, pd * 0.5 + 0.004, 0.0026, lawn);
        k.patch(dx - drive_w * 0.5, dx + drive_w * 0.5, zf - 0.03, pd * 0.5 + 0.004, 0.0034, drive);
    }
    let garage = |k: &mut Kit, z0: f32, z1: f32| {
        let g = Range::rect(dx - drive_w * 0.5 + 0.0006, dx + drive_w * 0.5 - 0.0006, z0, z1);
        let y0 = -k.drop(&g);
        let face = [Opening { u: drive_w * 0.5 - 0.0006, y: FOOT, w: 0.0108, h: 0.0118, kind: Open::Leaves { paint: shade(walls, 1.05), round: false } }];
        let front = if k.far() { Side::PLAIN } else { Side::with(&face) };
        k.walls(&g, y0, STOREY * 0.75, &wall, [front, Side::PLAIN, Side::PLAIN, Side::PLAIN], true, true);
        k.roof(&g, &RoofSpec { h: STOREY * 0.75, rise: 0.006, eaves: 0.0024, verge: 0.002, ends: [End::Gable, End::Gable], col: tile, gable: walls, thick: 0.0012 });
    };
    match t.built {
        ..=1945 => {
            let d = pd * 0.46;
            let body = Range::rect(hx0, hx1, zf - d, zf);
            let y0 = -k.drop(&body);
            let h = STOREY * 2.0;
            let mut front = Vec::new();
            if !k.far() {
                front.push(door(w * 0.72, paint));
                front.push(window(w * 0.72, 1, 0.0066));
                front.push(window(w * 0.26, 1, 0.0086));
            }
            let end = [window(d * 0.5, 1, 0.0062)];
            let end = if k.far() { Side::PLAIN } else { Side::with(&end) };
            k.walls(&body, y0, h, &wall, [Side::with(&front), end, Side::PLAIN, end], true, true);
            k.roof(&body, &RoofSpec { h, rise: d * 0.42, eaves: 0.0034, verge: 0.0, ends: [End::Hip, End::Hip], col: tile, gable: walls, thick: 0.0012 });
            k.bay(&body, 0, w * 0.26, 0.014, 0.004, y0, FOOT + 0.015, &wall, tile);
            k.stack(cx + w * 0.3, zf - d * 0.5, 0.006, 0.006, h + 0.006, h + d * 0.42 + 0.006, *pal.brick.chimney, 2);
            if t.built >= 1930 {
                garage(&mut k, zf - d * 0.9, zf - d * 0.2);
            }
        }
        1946..=1975 => {
            // a bungalow: the garage under the same long roof
            let d = pd * 0.42;
            let (x0, x1) = if left { (-pw * 0.5 + 0.003, pw * 0.5 - 0.004) } else { (-pw * 0.5 + 0.004, pw * 0.5 - 0.003) };
            let body = Range::rect(x0, x1, zf - d, zf);
            let y0 = -k.drop(&body);
            let h = STOREY * 0.95;
            let len = x1 - x0;
            let gu = if left { drive_w * 0.5 + 0.001 } else { len - drive_w * 0.5 - 0.001 };
            let mut front = Vec::new();
            if !k.far() {
                front.push(Opening { u: gu, y: FOOT, w: 0.0112, h: 0.0118, kind: Open::Leaves { paint: *pal.walls.limewash, round: false } });
                let mid = if left { len * 0.55 } else { len * 0.45 };
                front.push(door(mid, paint));
                front.push(Opening { u: if left { len * 0.8 } else { len * 0.2 }, y: 0.0062, w: 0.016, h: 0.0072, kind: Open::Casement });
            }
            let low = if r(6) < 0.5 { pal.walls.limewash.clone() } else { pal.walls.sage.clone() };
            let wall2 = Wall { col: if r(1) < 0.5 { walls } else { *low }, ..wall };
            k.walls(&body, y0, h, &wall2, [Side::with(&front), Side::PLAIN, Side::PLAIN, Side::PLAIN], true, true);
            k.roof(&body, &RoofSpec { h, rise: d * 0.26, eaves: 0.0048, verge: 0.0, ends: [End::Hip, End::Hip], col: tile, gable: wall2.col, thick: 0.0012 });
            k.stack(cx, zf - d * 0.45, 0.0055, 0.0055, h + 0.003, h + d * 0.26 + 0.004, *pal.brick.chimney, 1);
        }
        _ => {
            // a wing with its gable to the road, the range behind, the garage beside
            let d = pd * 0.4;
            let body = Range::rect(hx0, hx1, zf - d - 0.006, zf - 0.006);
            let ww = w * 0.48;
            let (wx0, wx1) = if left { (hx1 - ww, hx1) } else { (hx0, hx0 + ww) };
            let wing = Range { cx: (wx0 + wx1) * 0.5, cz: zf - d * 0.5, len: d, dep: ww, turn: 3 };
            let y0 = -k.drop(&body);
            let h = STOREY * 2.0;
            let mut front = Vec::new();
            let mut wf = Vec::new();
            if !k.far() {
                front.push(door(if left { 0.008 } else { w - 0.008 }, paint));
                front.push(window(if left { 0.008 } else { w - 0.008 }, 1, 0.0062));
                wf.push(window(ww * 0.5, 0, 0.0092));
                wf.push(window(ww * 0.5, 1, 0.0072));
            }
            k.walls(&body, y0, h, &wall, [Side::with(&front), Side::PLAIN, Side::PLAIN, Side::PLAIN], true, true);
            k.roof(&body, &RoofSpec { h, rise: d * 0.36, eaves: 0.003, verge: 0.0024, ends: [End::Gable, End::Gable], col: tile, gable: walls, thick: 0.0012 });
            let wing_box = Range::rect(wx0, wx1, zf - d * 0.6, zf);
            let wall_w = Wall { col: if r(7) < 0.5 { pal.walls.town_house(r(8)) } else { walls }, ..wall };
            // the wing's face to the road is the face toward +z: with turn 3, side 3 of the rect range
            let _ = wing;
            k.walls(&wing_box, y0, h, &wall_w, [Side::with(&wf), Side::PLAIN, Side::NONE, Side::PLAIN], true, true);
            let wr = Range { cx: (wx0 + wx1) * 0.5, cz: zf - d * 0.3, len: d * 0.6, dep: ww, turn: 3 };
            k.roof(&wr, &RoofSpec { h, rise: ww * 0.5, eaves: 0.003, verge: 0.0026, ends: [End::Gable, End::Valley(0.004)], col: tile, gable: wall_w.col, thick: 0.0012 });
            garage(&mut k, zf - d - 0.004, zf - 0.006);
        }
    }
    // the car on the drive, a tree behind
    if !k.far() {
        for i in 0..t.cars.min(2) {
            let col = paintwork(pal, r(20 + i as i32));
            let (at, way) = (k.pt(dx, 0.0034, pd * 0.5 - 0.008 - i as f32 * 0.013), k.dir(0.0, 0.0, 1.0));
            car(k.solid, at, way, col, pal);
        }
    }
    let tx = if left { pw * 0.3 } else { -pw * 0.3 };
    let p = k.pt(tx, 0.0, -pd * 0.5 + 0.008);
    hedge_tree(&mut m.green, &site.moved(p, 0.0), 0.3 + 0.12 * r(9), r(10) * 3.0, r(11) * 6.0, pal);
    let _ = Vec3::ZERO;
}
