use glam::Vec3;

use crate::colour::shade;
use crate::kit::Detail;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// A lighthouse: a tapering tower in bands of white and red, a lamp room and
/// a red cap. From the play distance it has the gallery round its lamp
/// room; close to, the gallery's rail, the bars of the lamp room's glass, a
/// door at its foot and a vane on its cap.
pub fn lighthouse(solid: &mut MeshBuf, site: &Site, detail: Detail, pal: &Palette) {
    let p = site.origin;
    let white = *pal.walls.lighthouse_white;
    let red = *pal.walls.lighthouse_red;
    let (far, near) = (detail == Detail::Far, detail == Detail::Near);
    let sides = if far { 6 } else { 8 };
    for i in 0..4 {
        let y = i as f32 * 0.024;
        let r0 = 0.016 - 0.0012 * i as f32;
        solid.frustum(p + Vec3::Y * y, r0, r0 - 0.0012, 0.024, sides, if i % 2 == 0 { white } else { red }, None);
    }
    solid.frustum(p + Vec3::Y * 0.096, 0.012, 0.012, 0.014, sides, *pal.glass.lighthouse_lamp, None);
    solid.frustum(p + Vec3::Y * 0.11, 0.014, 0.0, 0.014, sides, red, None);
    if far {
        return;
    }
    // the gallery round the lamp room
    let iron = *pal.iron.plate;
    solid.frustum_turned(p + Vec3::Y * 0.0946, 0.0152, 0.0152, 0.0014, sides, 0.0, 0.0, shade(white, 0.86), Some(shade(white, 0.94)));
    if !near {
        return;
    }
    let q = site.turn();
    let ring = |i: usize, r: f32, y: f32| {
        let a = i as f32 / 8.0 * std::f32::consts::TAU;
        p + Vec3::new(a.cos() * r, y, a.sin() * r)
    };
    for i in 0..8 {
        // the rail and its standards, and the bars between the lamp room's panes
        solid.beam(ring(i, 0.0148, 0.096), ring(i, 0.0148, 0.1), 0.0006, iron);
        solid.beam(ring(i, 0.0148, 0.1), ring(i + 1, 0.0148, 0.1), 0.0006, iron);
        solid.beam(ring(i, 0.0121, 0.096), ring(i, 0.0121, 0.11), 0.0007, shade(red, 0.8));
    }
    // the door at its foot, toward the front, and the vane
    let door = *pal.timber.door;
    let at = |x: f32, y: f32| p + q * Vec3::new(x, y, 0.0159);
    solid.quad_toward([at(-0.0028, 0.0), at(0.0028, 0.0), at(0.0026, 0.0125), at(-0.0026, 0.0125)], [shade(door, 0.8), shade(door, 0.8), door, door], q * Vec3::Z);
    solid.beam(p + Vec3::Y * 0.124, p + Vec3::Y * 0.1305, 0.0007, iron);
    solid.beam(p + Vec3::Y * 0.129 + q * Vec3::new(-0.0026, 0.0, 0.0), p + Vec3::Y * 0.129 + q * Vec3::new(0.0026, 0.0, 0.0), 0.0006, iron);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_bands_a_lamp_and_a_cap() {
        let pal = Palette::standard();
        let build = |detail| {
            let mut b = MeshBuf::default();
            lighthouse(&mut b, &Site::level(Vec3::ZERO, 0.0), detail, &pal);
            assert!(b.is_sound());
            b
        };
        let (far, play, near) = (build(Detail::Far), build(Detail::Play), build(Detail::Near));
        assert_eq!(far.triangles(), 4 * 12 + 12 + 6);
        assert_eq!(play.triangles(), 4 * 16 + 16 + 8 + 24);
        assert!(near.triangles() > play.triangles());
        assert!((far.bounds().unwrap().1.y - 0.124).abs() < 1e-6 && (play.bounds().unwrap().1.y - 0.124).abs() < 1e-6);
    }
}
