use glam::Vec3;

use super::{Kind, Works};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::parts::smoking_chimney;
use crate::site::Site;

/// The heavy trades: an engine works (in red brick) or a forge, a rolling
/// mill, a steel rail mill or a converter shop (in sooty brick). A long wide
/// shed, longer with each tier, with a chimney behind it for each tier; a
/// converter shop has its two converters standing at one end.
pub fn heavy(solid: &mut MeshBuf, smoke: &mut MeshBuf, site: &Site, w: &Works, pal: &Palette) {
    let (p, yaw, tier) = (site.origin, site.yaw, w.tier as f32);
    let brick = *pal.brick.red;
    let len = 0.09 + 0.03 * tier;
    solid.block(p, len, 0.06, 0.04, yaw, if w.kind == Kind::EngineWorks { brick } else { *pal.brick.dark }, None, 0.03);
    solid.gable(p, len, 0.06, 0.04, 0.025, yaw, 0.004, *pal.slate.works, brick);
    for i in 0..(tier as usize).max(1) {
        smoking_chimney(solid, smoke, pal, site.at(-len * 0.4 + i as f32 * 0.03, -0.045), 0.12 + 0.02 * tier, 0.008, w.seed + i as f32);
    }
    if w.kind == Kind::Bessemer {
        for i in 0..2 {
            solid.ball(site.at(len * 0.5 + 0.02, -0.02 + i as f32 * 0.035) + Vec3::Y * 0.02, 0.014, 0.018, 7, 3, *pal.iron.converter, i as f32);
        }
    }
}
