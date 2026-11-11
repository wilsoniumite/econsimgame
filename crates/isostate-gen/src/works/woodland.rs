//! The trades of the woods: charcoal burning and the bloomery.

use glam::Vec3;

use super::kit::{Ctx, Face};

/// Charcoal burning: turf-covered clamps breathing thin smoke in a clearing,
/// one being built of cordwood and one raked out, the burners' hut and the
/// wood stacked ready. More clamps and a wider clearing as it grows.
pub fn charcoal_burning(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.charcoal_hearth, true);
    let r = 0.013 * g;
    let wanted = (1.6 * size.sqrt()).ceil().max(2.0) as usize;
    // the places a clamp may stand: a staggered grid inside the clearing, nearest the middle first
    let (rx, rz, step) = (hw - r - 0.006, hd - r - 0.006, r * 2.4);
    let mut spots: Vec<(f32, f32)> = Vec::new();
    let reach = (hw / step).ceil() as i32 + 1;
    for j in -reach..=reach {
        for i in -reach..=reach {
            let (x, z) = ((i as f32 + if j & 1 == 0 { 0.25 } else { -0.25 }) * step, j as f32 * step * 0.87 - hd * 0.12);
            // the middle of the clearing: the hut and the cordwood have its rim
            if (x / rx).powi(2) + (z / rz).powi(2) <= 0.56 {
                spots.push((x, z));
            }
        }
    }
    spots.sort_by(|a, b| (a.0 * a.0 + a.1 * a.1 * 1.4).total_cmp(&(b.0 * b.0 + b.1 * b.1 * 1.4)));
    if spots.is_empty() {
        spots.push((0.0, 0.0));
    }
    let n = wanted.min(spots.len());
    let turf = c.tone(*pal.ground.charcoal_clamp);
    for (k, &(x, z)) in spots.iter().take(n).enumerate() {
        let p = c.at(x, z);
        // most are burning under their turf; one in four is being built, one in four raked out
        let stage = if c.busy() { (k + (c.w.seed.u * 4.0) as usize) % 4 } else { 0 };
        match stage {
            2 if k > 0 => {
                c.solid.frustum_turned(p, r * 0.9, r * 0.25, r * 0.9, 7, 0.3, 0.01, *pal.timber.logs, Some(*pal.timber.planks));
            }
            3 if k > 0 => {
                c.patch(x, z, r * 1.05, r * 1.05, 7, 0, *pal.goods.coal);
                c.heap(x + r * 0.3, z, r * 0.5, r * 0.35, *pal.goods.coal);
            }
            _ => {
                c.solid.frustum_turned(p, r, r * 0.62, r * 0.55, 7, 0.0, 0.01, turf, None);
                c.solid.frustum_turned(p + Vec3::Y * r * 0.55, r * 0.62, 0.0, r * 0.32, 7, 0.0, 0.0, c.tone(*pal.ground.charcoal_clamp_top), None);
                c.plume(p + Vec3::Y * r * 0.85, 0.42);
            }
        }
    }
    // the burners' hut: poles and turf, at the edge
    let (hx, hz) = (rx * 0.78, rz * 0.5);
    let hut = c.at(hx, hz);
    let (hr, hh) = (0.009 * g, 0.02 * g);
    c.solid.frustum_turned(hut, hr, 0.0, hh, 6, 0.4, 0.01, c.tone(*pal.timber.logs), None);
    c.note_peak(hut + Vec3::Y * hh);
    // cordwood stacked round the back of the clearing
    if c.stocked() {
        let stacks = 1 + n / 3;
        for i in 0..stacks {
            let a = std::f32::consts::PI * (1.15 + 0.7 * (i as f32 + 0.5) / stacks as f32);
            c.pile(a.cos() * rx * 0.86, a.sin() * rz * 0.86, 0.026 * g, 0.007 * g, 0.008 * g, false, *pal.timber.logs);
        }
    }
}

/// A bloomery: a small shaft furnace under a lean-to, charcoal, ore and
/// cinder in heaps about it. More shafts in a row as it grows, and a hammer
/// house (with a wheel where there is water).
pub fn bloomery(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.yard_earth, false);
    c.wall(c.yard_wall(), 0.006, 0.02 * g);
    let stone = c.tone(*pal.stone.works);
    let roof = c.roofing();
    let hammer = size >= 2.5;
    let pitch = (0.03 * g).min(2.0 * hw - 0.03);
    let room = 2.0 * hw - 0.02 - if hammer { 0.04 * g } else { 0.0 };
    let n = c.fit(room - 0.008 * g, pitch, 8).min(size.sqrt().round().max(1.0) as usize);
    let (len, dep) = (n as f32 * pitch + 0.006 * g, (0.026 * g).min(2.0 * hd - 0.014 - 0.019 * g));
    let x0 = -hw + 0.01 + len * 0.5;
    let zs = -hd + 0.008 + 0.01 * g;
    // the shafts, and the shelter over the working floor in front of them
    for i in 0..n {
        let x = x0 - len * 0.5 + 0.003 * g + pitch * (i as f32 + 0.5);
        c.flue(x, zs, 0.036 * g, 0.0095 * g, 0.0065 * g, stone, c.throat(), 0.5);
    }
    let shelter = c.lean_to(x0, zs + 0.008 * g + dep * 0.5, len, dep, 0.013 * g, 0.027 * g, false, c.tone(*pal.timber.boards), roof);
    if !c.going_up() {
        c.panel(&shelter, Face::Front, -len * 0.4, len * 0.4, 0.0, 0.01 * g, false, *pal.smoke.soot);
    }
    let front = zs + 0.008 * g + dep;
    if hammer {
        let (hl, hdp, hh) = (0.034 * g, 0.028 * g, 0.024 * g);
        let (hx, hz) = (hw - 0.01 - hl * 0.5, -hd + 0.009 + hdp * 0.5);
        let s = c.shed(hx, hz, hl, hdp, hh, false, stone, roof);
        if c.water_power() {
            let r = 0.014 * g;
            let wz = hz + hdp * 0.5 + 0.004 * g;
            c.race((hw - 0.004, wz), (hx - r, wz), 0.006 * g);
            c.wheel(hx, wz, r, 0.005 * g, false, c.tone(*pal.timber.waterwheel));
        } else if !c.going_up() {
            c.panel(&s, Face::Front, -hl * 0.2, hl * 0.2, 0.0, hh * 0.7, true, *pal.smoke.soot);
        }
    }
    // charcoal and ore to hand, cinder tipped by the gate, blooms in a row
    if c.stocked() {
        let z = (front + hd - 0.008) * 0.5;
        let r = (0.008 * g).min((hd - 0.008 - front) * 0.45);
        if r > 0.003 {
            c.heap(-hw * 0.62, z, r, r * 0.8, *pal.goods.coal);
            c.heap(-hw * 0.2, z, r, r * 0.8, *pal.ground.spoil_ironstone);
            let k = size.max(0.5).powf(0.25);
            c.heap(hw * 0.58, z, (r * 1.2 * k).min(hw * 0.3).min(hd - 0.008 - z), r * 0.9 * k.min(2.0), *pal.ground.slag);
            c.stacks(hw * 0.12, z, hw * 0.3, r * 1.4, c.fit(hw * 0.3, 0.012 * g, 6), 1, 0.004 * g, *pal.iron.plate);
        }
    }
}
