use glam::Vec3;

use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// A market cross on its step, in the middle of a small town's market place.
pub fn market_cross(solid: &mut MeshBuf, site: &Site, pal: &Palette) {
    let m = site.origin;
    let stone = *pal.stone.market_cross;
    solid.block(m, 0.016, 0.016, 0.004, site.yaw, stone, Some(stone), 0.0);
    solid.beam(m, m + Vec3::Y * 0.03, 0.0035, stone);
    solid.beam(m + Vec3::new(-0.006, 0.024, 0.0), m + Vec3::new(0.006, 0.024, 0.0), 0.003, stone);
}

/// A market hall on stone pillars, in the middle of a big town's market place.
pub fn market_hall(solid: &mut MeshBuf, site: &Site, pal: &Palette) {
    let m = site.origin;
    let stone = *pal.stone.church;
    for (dx, dz) in [(-0.016, -0.01), (0.016, -0.01), (-0.016, 0.01), (0.016, 0.01)] {
        let q = site.turn();
        let o = q * Vec3::new(dx, 0.0, dz);
        solid.beam(m + o, m + o + Vec3::Y * 0.014, 0.004, stone);
    }
    solid.block(m + Vec3::Y * 0.014, 0.044, 0.03, 0.012, site.yaw, *pal.walls.market_hall, None, 0.0);
    solid.gable(m + Vec3::Y * 0.014, 0.044, 0.03, 0.012, 0.014, site.yaw, 0.003, *pal.roofs.tile_dark, *pal.walls.market_hall);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hall_stands_on_its_pillars_and_the_cross_is_a_cross() {
        let pal = Palette::standard();
        let site = Site::level(Vec3::ZERO, 0.0);
        let (mut cross, mut hall) = (MeshBuf::default(), MeshBuf::default());
        market_cross(&mut cross, &site, &pal);
        market_hall(&mut hall, &site, &pal);
        assert!(cross.is_sound() && hall.is_sound());
        // a step, a shaft and an arm
        assert_eq!(cross.triangles(), 10 + 8 + 8);
        assert!((cross.bounds().unwrap().1.y - 0.03).abs() < 1e-6);
        // four pillars under the hall
        assert_eq!(hall.triangles(), 4 * 8 + 8 + 6);
        assert!(hall.bounds().unwrap().1.y > 0.035);
    }
}
