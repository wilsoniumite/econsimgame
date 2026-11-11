//! Parts that several generators share: a plain gabled building, chimneys,
//! smoke, and the windows and doors of a house.

use glam::{Quat, Vec3};

use crate::colour::Rgba;
use crate::mesh::MeshBuf;
use crate::palette::Palette;

/// The height of one storey of a house, in map units (the relief and the
/// buildings are both drawn about three times life size).
pub const STOREY: f32 = 0.024;

/// A plain building: four walls and a steep pitched roof, the ridge along its width.
#[allow(clippy::too_many_arguments)]
pub fn gabled(b: &mut MeshBuf, p: Vec3, w: f32, d: f32, storeys: f32, yaw: f32, wall: Rgba, roof: Rgba) {
    let h = STOREY * storeys;
    b.block(p, w, d, h, yaw, wall, None, 0.03);
    b.gable(p, w, d, h, d * 0.55, yaw, 0.004, roof, wall);
}

/// A round brick chimney standing on the ground at `p`, `h` tall, with a sooty top.
pub fn chimney(b: &mut MeshBuf, pal: &Palette, p: Vec3, h: f32, r: f32) {
    b.frustum(p, r, r * 0.8, h, 6, *pal.brick.chimney, Some(*pal.smoke.soot));
}

/// A chimney with a thin plume of smoke leaning downwind: a house fire's,
/// small beside a works'.
pub fn smoking_chimney(b: &mut MeshBuf, smoke: &mut MeshBuf, pal: &Palette, p: Vec3, h: f32, r: f32, seed: f32) {
    chimney(b, pal, p, h, r);
    crate::works::plume(smoke, p + Vec3::Y * h, 0.34 + r * 12.0, seed * 2.4, *pal.smoke.puff);
}

/// Windows and doors on both long faces: two windows a storey for each house in
/// the block, and a door in place of one on the ground floor of the front.
#[allow(clippy::too_many_arguments)]
pub fn windows(b: &mut MeshBuf, pal: &Palette, p: Vec3, w: f32, d: f32, storeys: f32, yaw: f32, units: usize, door: bool) {
    let q = Quat::from_rotation_y(yaw);
    let glass = *pal.glass.window;
    let wood = *pal.timber.door;
    let unit = w / units as f32;
    let (ww, wh) = ((unit * 0.16).min(0.007), 0.0075);
    for side in [-1.0f32, 1.0] {
        let z = side * (d * 0.5 + 0.0006);
        for u in 0..units {
            let cx = -w * 0.5 + unit * (u as f32 + 0.5);
            for s in 0..(storeys as usize) {
                let y = STOREY * s as f32 + 0.0125;
                for (k, dx) in [-unit * 0.24, unit * 0.24].into_iter().enumerate() {
                    let (x0, x1, y0, y1, col) = if s == 0 && door && k == 1 && side > 0.0 {
                        (cx + dx - ww * 0.5, cx + dx + ww * 0.5, 0.0005, 0.0145, wood)
                    } else {
                        (cx + dx - ww * 0.5, cx + dx + ww * 0.5, y - wh * 0.5, y + wh * 0.5, glass)
                    };
                    let c = [Vec3::new(x0, y0, z), Vec3::new(x1, y0, z), Vec3::new(x1, y1, z), Vec3::new(x0, y1, z)].map(|v| p + q * v);
                    if side > 0.0 {
                        b.quad(c[0], c[1], c[2], c[3], col);
                    } else {
                        b.quad(c[0], c[3], c[2], c[1], col);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_house_has_two_windows_a_storey_on_each_face_and_one_door() {
        let pal = Palette::standard();
        let mut b = MeshBuf::default();
        windows(&mut b, &pal, Vec3::ZERO, 0.05, 0.036, 2.0, 0.0, 1, true);
        // two faces, two storeys, two openings each
        assert_eq!(b.triangles(), 2 * 2 * 2 * 2);
        let doors = b.col.iter().filter(|c| **c == *pal.timber.door).count() / 4;
        assert_eq!(doors, 1);
        // the door is on the front, which is +z, and reaches the ground
        let door_at = b.col.iter().position(|c| *c == *pal.timber.door).unwrap();
        assert!(b.pos[door_at][2] > 0.0 && b.pos[door_at][1] < 0.001);
        assert_eq!(b.nrm[door_at], [0.0, 0.0, 1.0]);
    }

    #[test]
    fn smoke_goes_to_its_own_buffer() {
        let pal = Palette::standard();
        let (mut b, mut smoke) = (MeshBuf::default(), MeshBuf::default());
        smoking_chimney(&mut b, &mut smoke, &pal, Vec3::ZERO, 0.1, 0.01, 1.0);
        assert!(!b.is_empty() && !smoke.is_empty());
        // the plume rises from the chimney's top
        assert!(smoke.bounds().unwrap().0.y >= 0.1 && smoke.bounds().unwrap().1.y > 0.11);
        assert!(b.bounds().unwrap().1.y <= 0.1 + 1e-6);
    }
}
