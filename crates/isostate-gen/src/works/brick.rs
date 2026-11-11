//! The brickworks.

use glam::Vec3;

use super::kit::{Ctx, Face};

/// A brickworks: bottle kilns, more of them as it grows; open drying sheds;
/// bricks in the stack; the clay pit beside. A big works founded late has
/// a long continuous kiln with one tall chimney, and fewer bottle kilns.
pub fn brickworks(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.yard_earth, false);
    c.wall(*pal.timber.fence, 0.005, 0.03 * g);
    let brick = c.tone(*pal.brick.red);
    let continuous = c.w.founded >= 1860 && size >= 8.0;
    let wanted = (1.2 * size.sqrt()).ceil().max(1.0) as usize;
    let (r, pitch) = (0.015 * g, 0.036 * g);
    // the kilns and sheds have the left of the yard, the clay pit the right
    let kw = 2.0 * hw * 0.62 - 0.012;
    let per = c.fit(kw, pitch, 8);
    let rows = (if continuous { (wanted / 3).max(1) } else { wanted }).div_ceil(per).min(2);
    let kilns = (if continuous { (wanted / 3).max(1) } else { wanted }).min(rows * per);
    for k in 0..kilns {
        let (x, z) = (-hw + 0.008 + pitch * ((k % per) as f32 + 0.5), -hd + 0.008 + r * 1.15 + (k / per) as f32 * pitch);
        let p = c.at(x, z);
        let whole = !c.going_up() && !c.derelict();
        c.solid.frustum_turned(p, r, r * 1.15, 0.018 * g, 7, 0.0, 0.02, brick, None);
        if c.near() && !c.going_up() {
            // the iron band round the kiln's shoulder, and its firing holes round the foot
            let band = crate::colour::shade(brick, 0.78);
            c.solid.frustum_turned(p + Vec3::Y * 0.0172 * g, r * 1.15 + 0.0005, r * 1.15 + 0.0005, 0.0012, 7, 0.0, 0.0, band, None);
            for i in 0..7 {
                let a = (i as f32 + 0.5) / 7.0 * std::f32::consts::TAU;
                let (o, s) = (Vec3::new(a.cos(), 0.0, a.sin()), Vec3::new(-a.sin(), 0.0, a.cos()));
                let at = |w: f32, y: f32| p + o * (r * 1.04 * (std::f32::consts::PI / 7.0).cos() + 0.0006) + s * w + Vec3::Y * y;
                c.solid.quad_toward([at(-0.0016, 0.0), at(0.0016, 0.0), at(0.0016, 0.0042), at(-0.0016, 0.0042)], [*pal.smoke.soot; 4], o);
            }
        }
        if !c.going_up() {
            let (top, neck) = (if whole { 0.034 * g } else { 0.02 * g }, if whole { r * 0.38 } else { r * 0.7 });
            c.solid.frustum_turned(p + Vec3::Y * 0.018 * g, r * 1.15, neck, top, 7, 0.0, 0.0, brick, Some(*pal.smoke.soot));
            // they are fired in turn
            if k % 2 == 0 {
                c.plume(p + Vec3::Y * (0.018 * g + top), 0.6);
            }
        }
    }
    let mut front = -hd + 0.008 + rows as f32 * pitch;
    if continuous {
        // the continuous kiln: a ring of chambers under one long roof, and its chimney
        let (len, dep, h) = ((kw - 0.02 * g).min(0.2 * g), 0.045 * g, 0.016 * g);
        let (x, z) = (-hw + 0.008 + len * 0.5, front + 0.006 + dep * 0.5);
        let s = c.shed_pitched(x, z, len, dep, h, 0.3, false, brick, c.tone(*pal.roofs.tile_dark));
        if !c.going_up() {
            let wickets = c.fit(len, 0.014 * g, 30);
            for face in [Face::Front, Face::Back] {
                for i in 0..wickets {
                    let cx = -len * 0.5 + len * (i as f32 + 0.5) / wickets as f32;
                    c.mouth(&s, face, cx - 0.003 * g, cx + 0.003 * g, 0.0, h * 0.55, true);
                }
            }
        }
        c.chimney(x + len * 0.5 + 0.014 * g, z, 0.19 * g, 0.009 * g, 1.3);
        front = z + dep * 0.5;
    }
    // drying sheds, and bricks in the stack beside them
    let z0 = front + 0.006;
    let depth = hd - 0.012 - z0;
    let (sl, sd) = ((0.06 * g).min(kw * 0.55), 0.018 * g);
    let shed_rows = if depth >= sd + 0.004 { c.fit(depth, sd + 0.006, 6) } else { 0 };
    let cols = c.fit(kw * 0.62, sl + 0.008, 3);
    for j in 0..shed_rows {
        for i in 0..cols {
            let x = -hw + 0.008 + sl * 0.5 + i as f32 * (sl + 0.008);
            c.open_shed(x, z0 + sd * 0.5 + j as f32 * (sd + 0.006), sl, sd, 0.011 * g, false, *pal.timber.headframe, c.tone(*pal.roofs.drying_shed));
        }
    }
    let bx0 = -hw + 0.008 + if shed_rows > 0 { cols as f32 * (sl + 0.008) } else { 0.0 };
    let bw = -hw + 0.008 + kw - bx0;
    if bw > 0.012 && depth > 0.008 {
        c.stacks(bx0 + bw * 0.5, z0 + depth * 0.5, bw, depth, c.fit(bw, 0.014 * g, 10), c.fit(depth, 0.011 * g, 10), 0.011 * g, *pal.goods.bricks);
    }
    // the clay pit
    let (px, prx, prz) = (-hw + 2.0 * hw * 0.81, 2.0 * hw * 0.165, hd * 0.78);
    c.patch(px, 0.0, prx, prz, 9, 0, *pal.ground.clay_face);
    c.patch(px + prx * 0.1, prz * 0.05, prx * 0.55, prz * 0.55, 7, 1, *pal.ground.clay_bottom);
    if c.stocked() {
        c.heap(px - prx * 0.55, prz * 0.5, (0.008 * g).min(prx * 0.3), 0.006 * g, *pal.ground.spoil_clay);
    }
}
