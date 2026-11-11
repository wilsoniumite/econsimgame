use glam::Vec3;

use super::Works;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// A brickworks: a row of bottle kilns, one more for each tier up to four,
/// and a long low drying shed in front.
pub fn brickworks(solid: &mut MeshBuf, site: &Site, w: &Works, pal: &Palette) {
    let brick = *pal.brick.red;
    for i in 0..(w.tier as usize + 1).min(4) {
        let o = site.at(-0.03 + i as f32 * 0.03, -0.02);
        solid.frustum(o, 0.016, 0.019, 0.02, 7, brick, None);
        solid.frustum(o + Vec3::Y * 0.02, 0.019, 0.006, 0.03, 7, brick, Some(*pal.smoke.soot));
    }
    solid.block(site.at(0.0, 0.03), 0.1, 0.025, 0.012, site.yaw, *pal.timber.boards, Some(*pal.roofs.drying_shed), 0.02);
}
