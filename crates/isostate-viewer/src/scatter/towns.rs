//! Towns: a church on the market square, a station once the railway comes, a
//! lighthouse for a harbour, and the houses, which go up in rows along the
//! streets before the blocks behind fill in.

use super::*;

pub(super) fn landmarks(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, houses: &mut Builder, plots: &mut std::collections::HashMap<usize, Vec<(f32, f32)>>) {
    // each town: a church on its market square, a station once the railway arrives,
    // and a lighthouse if it is a harbour
    let nt = r.n_towns.min(r.header.towns.len());
    for t in 0..nt {
        if r.town_active[k * r.n_towns + t] == 0 {
            continue;
        }
        let node = r.header.towns[t].node;
        let (x, z) = cen[node];
        let yaw = (world::hash2(t as i32, 1, 5) * 4.0).floor() * std::f32::consts::FRAC_PI_2;
        let big = r.town_urban[k * r.n_towns + t] > 12000.0;
        church(houses, ground(g, x, z), yaw, big);
        plots.entry(node).or_default().push((x, z));
        if r.trans[k * r.n + node] & 4 != 0 {
            let q = bevy::math::Quat::from_rotation_y(yaw);
            let o = q * Vec3::new(0.0, 0.0, 0.2);
            station(houses, ground(g, x + o.x, z + o.z), yaw);
            plots.entry(node).or_default().push((x + o.x, z + o.z));
        }
        let (cx, cy) = ((node % r.w) as isize, (node / r.w) as isize);
        let mut sea: Option<(f32, f32)> = None;
        for (dx, dy) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)] {
            let (nx, ny) = (cx + dx, cy + dy);
            if nx >= 0 && ny >= 0 && (nx as usize) < r.w && (ny as usize) < r.h && r.terrain[ny as usize * r.w + nx as usize] == 0 {
                sea = Some((dx as f32, dy as f32));
                break;
            }
        }
        if let Some((dx, dy)) = sea {
            // walk toward the sea until the ground is nearly at the waterline
            let (mut lx, mut lz) = (x, z);
            for _ in 0..40 {
                let (nx, nz) = (lx + dx * 0.025, lz + dy * 0.025);
                if world::height_at(g, nx, nz) < 0.03 {
                    break;
                }
                lx = nx;
                lz = nz;
            }
            lighthouse(houses, ground(g, lx, lz));
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn houses(r: &Replay, g: &Ground, slots: &Slots, cen: &[(f32, f32)], k: usize, near_plot: &dyn Fn(f32, f32) -> bool, houses: &mut Builder, smoke: &mut Builder) {
    // town houses: rows along the streets first, then the blocks behind fill in
    let density = |c: usize| -> f32 {
        let off = k * r.n + c;
        if r.use_[off] == 10 { r.inten[off] as f32 / 255.0 } else { 0.45 }
    };
    let market: Vec<(f32, f32)> = (0..r.n_towns.min(r.header.towns.len()))
        .filter(|&t| r.town_active[k * r.n_towns + t] > 0)
        .map(|t| cen[r.header.towns[t].node])
        .collect();
    let near_market = |x: f32, z: f32| market.iter().any(|&(mx, mz)| (mx - x).powi(2) + (mz - z).powi(2) < 0.11 * 0.11);
    let mut spots: Vec<HouseSpot> = Vec::new();
    for (c, m, lv) in crate::network::road_segments(r, k) {
        let (a, b) = (cen[c], cen[m]);
        let (dx, dz) = (b.0 - a.0, b.1 - a.1);
        let len = (dx * dx + dz * dz).sqrt().max(1e-4);
        let (ux, uz) = (dx / len, dz / len);
        let along = (-uz).atan2(ux);
        let off = if lv == 3 { 0.05 } else { 0.044 };
        let n = (len / 0.068).floor().max(1.0) as usize;
        let dir = if m == c + 1 { 0 } else if m == c + r.w { 1 } else if m == c + r.w + 1 { 2 } else { 3 };
        for i in 0..n {
            let t = (i as f32 + 0.5) / n as f32;
            if !(0.1..=0.9).contains(&t) {
                continue;
            }
            for (si, side) in [(0, -1.0f32), (1, 1.0)] {
                let id = (((c * 4 + dir) * 32 + i.min(31)) * 2 + si) as i32;
                let jit = (world::hash2(id, 1, 21) - 0.5) * 0.016;
                let x = a.0 + dx * t + -uz * off * side + ux * jit;
                let z = a.1 + dz * t + ux * off * side + uz * jit;
                if world::height_at(g, x, z) < 0.05 || near_market(x, z) || near_plot(x, z) {
                    continue;
                }
                let (wx, wy) = world::warp(x, z);
                let wc = (wy.floor().clamp(0.0, r.h as f32 - 1.0) as usize) * r.w + wx.floor().clamp(0.0, r.w as f32 - 1.0) as usize;
                if world::pick_class(r, k, wc, world::sub_uniform(g, x, z) * 255.0) != world::URBAN {
                    continue;
                }
                let (u, v) = (world::hash2(id, 3, 21), world::hash2(id, 4, 21));
                if u > 0.35 + 0.65 * density(wc) {
                    continue;
                }
                // fronts to the street: the ridge runs along it
                let yaw = along + if side < 0.0 { std::f32::consts::PI } else { 0.0 } + (v - 0.5) * 0.06;
                spots.push(HouseSpot { x, z, yaw, rank: u * 0.7, u, v, w: 0.048 + 0.016 * v, d: 0.036 + 0.006 * u });
            }
        }
    }
    for s in &slots.houses {
        let c = s.wcell as usize;
        if world::pick_class(r, k, c, s.sub) != world::URBAN || near_plot(s.x, s.z) || near_market(s.x, s.z) {
            continue;
        }
        if s.u > 0.25 + 0.7 * density(c) {
            continue;
        }
        let yaw = (s.yaw / std::f32::consts::FRAC_PI_2).round() * std::f32::consts::FRAC_PI_2 + (s.v - 0.5) * 0.12;
        spots.push(HouseSpot { x: s.x, z: s.z, yaw, rank: 0.25 + 0.75 * s.u, u: s.u, v: s.v, w: 0.045 + 0.02 * s.v, d: 0.035 + 0.008 * s.u });
    }
    // the earliest spots to fill take the ground; later ones only fill what is free
    spots.sort_by(|a, b| a.rank.partial_cmp(&b.rank).unwrap_or(std::cmp::Ordering::Equal));
    let mut taken: std::collections::HashMap<(i32, i32), Vec<(f32, f32)>> = Default::default();
    for s in spots {
        let key = ((s.x / 0.05).floor() as i32, (s.z / 0.05).floor() as i32);
        let clash = (-1..=1).any(|dy| (-1..=1).any(|dx| taken.get(&(key.0 + dx, key.1 + dy)).is_some_and(|v| v.iter().any(|&(x, z)| (x - s.x).powi(2) + (z - s.z).powi(2) < 0.047 * 0.047))));
        if clash {
            continue;
        }
        taken.entry(key).or_default().push((s.x, s.z));
        let c = (s.z.floor().clamp(0.0, r.h as f32 - 1.0) as usize) * r.w + s.x.floor().clamp(0.0, r.w as f32 - 1.0) as usize;
        let dense = density(c);
        let p = ground(g, s.x, s.z);
        let storeys = 1.0 + ((dense > 0.6 && s.v > 0.4) as u8 as f32) + ((dense > 0.85 && s.v > 0.75) as u8 as f32);
        let wall = hex(WALLS[(s.v * 6.0) as usize % 6]);
        let roof = hex(ROOFS[(s.u * 13.0) as usize % 5]);
        house(houses, p, s.w, s.d, storeys, s.yaw, wall, roof);
        if s.u < 0.2 {
            let cp = p + bevy::math::Quat::from_rotation_y(s.yaw) * Vec3::new(s.w * 0.3, 0.0, 0.0);
            if s.v > 0.8 {
                smoking_chimney(houses, smoke, cp, 0.024 * storeys + 0.03, 0.004, s.u * 7.0);
            } else {
                chimney(houses, cp, 0.024 * storeys + 0.03, 0.004);
            }
        }
    }

}
