use glam::Vec3;

use super::Works;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::parts::smoking_chimney;
use crate::site::Site;

/// A powered mill: a tall brick block, longer with each tier, three lit rows
/// of windows down its front, and a tall chimney at one end.
pub fn textile_mill(solid: &mut MeshBuf, smoke: &mut MeshBuf, site: &Site, w: &Works, pal: &Palette) {
    let (p, yaw, tier) = (site.origin, site.yaw, w.tier as f32);
    let q = site.turn();
    let brick = *pal.brick.red;
    let len = 0.08 + 0.025 * tier;
    solid.block(p, len, 0.04, 0.075, yaw, brick, None, 0.03);
    // rows of windows
    for row in 0..3 {
        let y = 0.016 + row as f32 * 0.022;
        let wp = p + q * Vec3::new(0.0, y, 0.0201);
        solid.block(wp - Vec3::Y * 0.0, len * 0.9, 0.001, 0.008, yaw, *pal.glass.mill_lit, None, 0.0);
    }
    solid.gable(p, len, 0.04, 0.075, 0.016, yaw, 0.003, *pal.slate.works, brick);
    smoking_chimney(solid, smoke, pal, site.at(len * 0.5 + 0.015, 0.0), 0.16 + 0.02 * tier, 0.009, w.seed);
}
