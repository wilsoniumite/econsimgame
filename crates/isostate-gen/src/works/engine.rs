//! The engine works.

use glam::Vec3;

use super::kit::{Ctx, Face, STOREY};

/// An engine works: erecting shops with tall arched doors, side by side
/// under one gable each or, once there is iron and glass, under a saw-tooth
/// of north lights; a foundry with its cupola; a boiler and a flywheel
/// waiting in the yard. More and longer rows of shops as it grows, a crane,
/// and an office by the gate.
pub fn engine_works(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.yard_setts, false);
    c.wall(c.yard_wall(), 0.008, 0.026 * g);
    let (wall, roof) = (c.masonry(false), c.roofing());
    let plate = c.tone(*pal.iron.plate);
    let saw = c.late();
    let (bay, dep, h) = (if saw { 0.02 * g } else { 0.024 * g }, 0.055 * g, 0.036 * g);
    let foundry = size >= 0.8;
    let (fl, fd) = (0.034 * g, 0.045 * g);
    let room = 2.0 * hw - 0.016 - if foundry { fl + 0.006 } else { 0.014 * g };
    let wanted = (3.0 * size.sqrt()).ceil().max(2.0) as usize;
    let per = c.fit(room, bay, 16).min(wanted);
    let pitch = dep + 0.014 * g;
    let rows = c.fit(2.0 * hd - 0.016 - 0.3 * hd, pitch, 4).min(wanted.div_ceil(per));
    let mut left = wanted.min(rows * per);
    let mut front = -hd;
    for j in 0..rows {
        let n = left.min(per);
        left -= n;
        let z = -hd + 0.008 + dep * 0.5 + j as f32 * pitch;
        let x0 = -hw + 0.008;
        if saw {
            let s = c.sawtooth(x0 + n as f32 * bay * 0.5, z, n as f32 * bay, dep, h, n, false, wall, roof);
            if !c.going_up() {
                for i in (0..n).step_by(2) {
                    let cx = -(n as f32) * bay * 0.5 + bay * (i as f32 + 0.5);
                    c.doors(&s, Face::Front, cx - bay * 0.3, cx + bay * 0.3, 0.0, h * 0.62, true, *pal.timber.door);
                }
            }
        } else {
            for i in 0..n {
                let s = c.shed(x0 + bay * (i as f32 + 0.5), z, dep, bay, h, true, wall, roof);
                if !c.going_up() {
                    c.doors(&s, Face::EndB, -bay * 0.28, bay * 0.28, 0.0, h * 0.7, true, *pal.timber.door);
                }
            }
        }
        front = z + dep * 0.5;
    }
    // the foundry and its cupola, under a spark arrester
    let (cx, cz) = if foundry {
        let (x, z) = (hw - 0.008 - fl * 0.5, -hd + 0.008 + fd * 0.5);
        c.shed(x, z, fd, fl, 0.028 * g, true, c.masonry(true), roof);
        (x, z + fd * 0.5 + 0.009 * g)
    } else {
        (hw - 0.008 - 0.007 * g, -hd + 0.008 + 0.007 * g)
    };
    let top = c.flue(cx, cz, 0.075 * g, 0.006 * g, 0.005 * g, plate, *pal.smoke.soot, 0.7);
    if !c.going_up() && !c.derelict() {
        c.solid.frustum_turned(top + Vec3::Y * 0.004 * g, 0.0105 * g, 0.0, 0.009 * g, 6, 0.0, 0.0, plate, None);
    }
    // work waiting in the yard: a boiler, a flywheel, castings; a crane; the office
    let z0 = front + 0.008;
    let d = hd - 0.012 - z0;
    if d > 0.008 {
        let z = z0 + d * 0.5;
        if c.stocked() {
            c.pile(-hw * 0.6, z, (0.024 * g).min(hw * 0.5), 0.008 * g, 0.008 * g, false, plate);
            c.wheel(-hw * 0.12, z, (0.011 * g).min(d * 0.5), 0.004 * g, false, plate);
        }
        if size >= 5.0 {
            c.sheer_legs(hw * 0.18, z, 0.05 * g);
            c.stacks(-hw * 0.45, z + d * 0.2, hw * 0.7, d * 0.4, c.fit(hw * 0.7, 0.016 * g, 10), 1, 0.005 * g, plate);
        }
        if size >= 10.0 && d > 0.03 * g {
            let s = c.shed(hw * 0.55, hd - 0.012 - 0.011 * g, 0.04 * g, 0.022 * g, 2.0 * STOREY, false, wall, roof);
            c.windows(&s, 0, 2, true);
        }
    }
}
