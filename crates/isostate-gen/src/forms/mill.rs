//! A mill of the railway years kept and turned to flats. The simulation
//! knows the mill (its method, its founding, its size, the year it
//! closed); what it would need to know to turn it is that its ground came
//! to be worth more as rooms than as a works' site (`ground` and `sites`,
//! the run to 2050's eighth stage).

use crate::colour::shade;
use super::{flat_roof, Tones};
use crate::hash::Seed;
use crate::houses::Detail;
use crate::kit::{End, Kit, Open, Opening, Range, RoofSpec, Side, Wall, STOREY};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::scale::BAY;
use crate::site::Site;
use crate::trees::{hedge_tree, Greenery};

#[derive(Clone, Copy, Debug)]
pub struct MillFlats {
    /// bays along it and storeys: the mill's own, from its size
    pub bays: usize,
    pub storeys: u8,
    /// the year it was turned to flats; None for the mill as it stood, closed
    pub turned: Option<i32>,
    /// standing closed (its windows broken), before it was turned
    pub closed: bool,
    pub detail: Detail,
    pub seed: Seed,
}

pub const MILL_DEPTH: f32 = 4.0 * BAY;

pub(crate) fn mill_flats(solid: &mut MeshBuf, flat: &mut MeshBuf, green: &mut Greenery, site: &Site, m: &MillFlats, pal: &Palette) {
    let t = Tones::of(pal);
    let r = |n: i32| m.seed.pick(n);
    let mut k = Kit::new(solid, site, pal, m.detail, 0.0);
    let bay = BAY * 1.1;
    let (len, dep) = (m.bays as f32 * bay, MILL_DEPTH);
    let (hl, hd) = (len * 0.5, dep * 0.5);
    let n = m.storeys as usize;
    // (a mill's storeys were tall, and are drawn at the scale's storey, as every building's)
    let h = n as f32 * STOREY;
    let sh = STOREY;
    let rg = Range::rect(-hl, hl, -hd, hd);
    let y0 = -k.drop(&rg);
    let brick = if r(1) < 0.5 { *pal.brick.red } else { *pal.brick.terrace_sooty };
    let turned = m.turned.is_some();
    // turned to flats its brick is cleaned of a century's soot
    let brick = if turned { shade(brick, 1.06) } else { shade(brick, 0.9) };
    let lit = !turned && !m.closed;
    let wall = Wall { col: brick, plinth: Some(*pal.stone.plinth), frame: if turned { t.concrete_dark } else { *pal.walls.window_frame }, sill: *pal.stone.sill };
    let windows = |len: f32, count: usize| -> Vec<Opening> {
        let mut v = Vec::new();
        for i in 0..count {
            let u = len * (i as f32 + 0.5) / count as f32;
            for s in 0..n {
                let kind = if turned { Open::Paned { cols: 2, rows: 3, round: false, lit: false } } else if m.closed && crate::hash::hash2(i as i32, s as i32, 5) < 0.3 { Open::Mouth { within: *pal.smoke.soot, round: false } } else { Open::Paned { cols: 3, rows: 4, round: false, lit: lit && crate::hash::hash2(i as i32, s as i32, 6) < 0.5 } };
                v.push(Opening { u, y: s as f32 * sh + 0.004, w: 0.0074, h: 0.0118, kind });
            }
        }
        v
    };
    let (long, short) = if k.far() { (Vec::new(), Vec::new()) } else { (windows(len, m.bays), windows(dep, 4)) };
    k.walls(&rg, y0, h, &wall, [Side::free(&long), Side::free(&short), Side::free(&long), Side::free(&short)], true, false);
    if !k.far() {
        // the stone bands at each floor
        for s in 1..n {
            let y = s as f32 * sh;
            for i in 0..4 {
                let l = rg.face(i).len;
                k.strip(&rg, i, 0.0, l, y - 0.0005, y - 0.0005 + super::thick(&k, 0.0012), 0.0005, *pal.stone.sill);
            }
        }
    }
    // the stair tower at one end, with its tank at the top
    let sx = hl - 0.012;
    let st = Range::rect(sx - 0.009, sx + 0.009, hd - 0.002, hd + 0.012);
    let tw = Wall { col: shade(brick, 0.96), ..wall };
    k.walls(&st, y0, h + 0.016, &tw, [Side::PLAIN, Side::PLAIN, Side::NONE, Side::PLAIN], true, false);
    k.roof(&st, &RoofSpec { h: h + 0.016, rise: 0.007, eaves: 0.0016, verge: 0.0, ends: [End::Hip, End::Hip], col: *pal.slate.works, gable: brick, thick: 0.001 });
    // the chimney, kept
    let c = k.pt(-hl - 0.02, 0.0, -hd + 0.01);
    k.solid.frustum_turned(c, 0.0068, 0.0042, h + 0.09, 8, 0.0, 0.01, shade(brick, 0.95), Some(*pal.smoke.stack_top));
    if !k.far() {
        k.solid.block(c, 0.016, 0.016, 0.012, site.yaw, shade(brick, 0.9), Some(shade(brick, 0.95)), 0.004);
    }
    match m.turned {
        None => {
            k.roof(&rg, &RoofSpec { h, rise: dep * 0.18, eaves: 0.0022, verge: 0.0016, ends: [End::Gable, End::Gable], col: shade(*pal.slate.works, 0.9), gable: brick, thick: 0.001 });
        }
        Some(_) => {
            // a storey of glass set back on the roof, its terrace railed
            flat_roof(&mut k, -hl, hl, -hd, hd, h, 0.0026, *pal.stone.sill, t.felt);
            let (px0, px1, pz0, pz1) = (-hl + 0.008, sx - 0.012, -hd + 0.004, hd - 0.008);
            let pent = Range::rect(px0, px1, pz0, pz1);
            let gw = Wall { col: t.concrete_dark, plinth: None, frame: *pal.modern.mullion, sill: t.concrete };
            let panes: Vec<Opening> = if k.far() { Vec::new() } else { super::rows(pent.len, n, n + 1, BAY - 0.0024, STOREY * 0.9 - 0.0036, 0.0016, Open::Glazed { glass: t.glass_tower, cols: 1, rows: 1 }, BAY) };
            let ends: Vec<Opening> = if k.far() { Vec::new() } else { super::rows(pent.dep, n, n + 1, BAY - 0.0024, STOREY * 0.9 - 0.0036, 0.0016, Open::Glazed { glass: t.glass_tower, cols: 1, rows: 1 }, BAY) };
            k.walls(&pent, h, h + STOREY * 0.9, &gw, [Side::free(&panes), Side::free(&ends), Side::free(&panes), Side::free(&ends)], false, false);
            flat_roof(&mut k, px0 - 0.002, px1 + 0.002, pz0 - 0.002, pz1 + 0.002, h + STOREY * 0.9, 0.0, t.concrete_dark, t.felt);
            if !k.far() {
                // balconies of steel and glass on the front
                let steel = *pal.iron.plate;
                for i in 0..m.bays {
                    if i % 2 == 1 {
                        continue;
                    }
                    let x = -hl + bay * (i as f32 + 0.5);
                    for s in 1..n {
                        let y = s as f32 * sh;
                        let st = super::thick(&k, 0.0009);
                        k.boxy(x - 0.005, x + 0.005, hd, hd + 0.004, y - st * 0.55, y + st * 0.45, steel, Some(steel));
                        super::pane_z(&mut k, x - 0.005, x + 0.005, hd + 0.0041, y + 0.0004, y + 0.0052, shade(t.glass_tower, 1.05), 1.0);
                    }
                }
            }
            // its yard: a car park and lawns, trees along its edge
            let lift = 0.0016;
            let p = |k: &Kit, x: f32, z: f32| k.pt(x, lift, z);
            flat.quad(p(&k, -hl - 0.03, hd + 0.05), p(&k, hl + 0.01, hd + 0.05), p(&k, hl + 0.01, hd + 0.003), p(&k, -hl - 0.03, hd + 0.003), t.verge);
            super::sheds::car_park(flat, &mut k, -hl * 0.6, hl * 0.6, hd + 0.014, hd + 0.048, 0.6, m.seed, &t);
            for i in 0..4 {
                let x = -hl + len * (i as f32 + 0.5) / 4.0;
                let q = k.pt(x, 0.0, hd + 0.054);
                hedge_tree(green, &site.moved(q, 0.0), 0.24, i as f32, i as f32 * 1.7, pal);
            }
        }
    }
}
