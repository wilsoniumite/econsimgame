use glam::Vec3;

use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// A parish church: a stone nave under slate with a west tower and a
/// spire. The site is the middle of the nave, which runs along the site's x
/// with the tower at its -x end; a big town's church is a third larger.
pub fn church(solid: &mut MeshBuf, site: &Site, big: bool, pal: &Palette) {
    let (p, yaw) = (site.origin, site.yaw);
    let stone = *pal.stone.church;
    let slate = *pal.slate.town;
    let k = if big { 1.35 } else { 1.0 };
    let q = site.turn();
    solid.block(p, 0.085 * k, 0.04 * k, 0.034 * k, yaw, stone, None, 0.03);
    solid.gable(p, 0.085 * k, 0.04 * k, 0.034 * k, 0.026 * k, yaw, 0.003, slate, stone);
    let tower = p + q * Vec3::new(-0.05 * k, 0.0, 0.0);
    solid.block(tower, 0.028 * k, 0.028 * k, 0.075 * k, yaw, stone, Some(stone), 0.03);
    solid.frustum(tower + Vec3::Y * 0.075 * k, 0.021 * k, 0.0, 0.07 * k, 4, slate, None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_spire_stands_over_the_town() {
        let pal = Palette::standard();
        let build = |big| {
            let mut b = MeshBuf::default();
            church(&mut b, &Site::level(Vec3::ZERO, 0.0), big, &pal);
            assert!(b.is_sound());
            b.bounds().unwrap()
        };
        let (small, big) = (build(false), build(true));
        // taller than any three-storey house with its roof (about 0.09)
        assert!(small.1.y > 0.14);
        assert!((big.1.y / small.1.y - 1.35).abs() < 0.01);
        // the tower is at the -x end
        assert!(-small.0.x > small.1.x);
    }
}
