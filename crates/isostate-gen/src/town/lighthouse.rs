use glam::Vec3;

use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// A lighthouse: a tapering tower in bands of white and red, a lamp room and a red cap.
pub fn lighthouse(solid: &mut MeshBuf, site: &Site, pal: &Palette) {
    let p = site.origin;
    let white = *pal.walls.lighthouse_white;
    let red = *pal.walls.lighthouse_red;
    for i in 0..4 {
        let y = i as f32 * 0.024;
        let r0 = 0.016 - 0.0012 * i as f32;
        solid.frustum(p + Vec3::Y * y, r0, r0 - 0.0012, 0.024, 8, if i % 2 == 0 { white } else { red }, None);
    }
    solid.frustum(p + Vec3::Y * 0.096, 0.012, 0.012, 0.014, 8, *pal.glass.lighthouse_lamp, None);
    solid.frustum(p + Vec3::Y * 0.11, 0.014, 0.0, 0.014, 8, red, None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_bands_a_lamp_and_a_cap() {
        let pal = Palette::standard();
        let mut b = MeshBuf::default();
        lighthouse(&mut b, &Site::level(Vec3::ZERO, 0.0), &pal);
        assert!(b.is_sound());
        assert_eq!(b.triangles(), 4 * 16 + 16 + 8);
        assert!((b.bounds().unwrap().1.y - 0.124).abs() < 1e-6);
    }
}
