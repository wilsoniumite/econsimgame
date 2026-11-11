use glam::Vec3;

use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// A railway station: a long low building under tile with a clock turret
/// through its roof, and a platform under a canopy along its front (+z).
pub fn station(solid: &mut MeshBuf, site: &Site, pal: &Palette) {
    let (p, yaw) = (site.origin, site.yaw);
    let q = site.turn();
    let (wall, roof, canopy) = (*pal.walls.station, *pal.roofs.tile_dark, *pal.roofs.station_canopy);
    solid.block(p, 0.09, 0.035, 0.03, yaw, wall, None, 0.03);
    solid.gable(p, 0.09, 0.035, 0.03, 0.018, yaw, 0.004, roof, wall);
    // platform canopy
    let c = p + q * Vec3::new(0.0, 0.0, 0.035);
    solid.block(c, 0.12, 0.03, 0.002, yaw, *pal.ground.station_platform, None, 0.0);
    solid.block(c + Vec3::Y * 0.022, 0.13, 0.034, 0.004, yaw, canopy, Some(canopy), 0.0);
    // clock turret
    solid.block(p + q * Vec3::new(0.0, 0.0, 0.0), 0.016, 0.016, 0.06, yaw, wall, Some(wall), 0.0);
    solid.frustum(p + Vec3::Y * 0.06, 0.013, 0.0, 0.022, 4, roof, None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_platform_is_along_the_front() {
        let mut b = MeshBuf::default();
        station(&mut b, &Site::level(Vec3::ZERO, 0.0), &Palette::standard());
        assert!(b.is_sound());
        let (lo, hi) = b.bounds().unwrap();
        assert!(hi.z > 0.05 && lo.z > -0.03);
        assert!((hi.y - 0.082).abs() < 1e-6);
    }
}
