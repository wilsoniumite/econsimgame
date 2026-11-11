use super::Works;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// A works of a kind the generators do not know: a plain brick shed under slate.
pub fn plain(solid: &mut MeshBuf, site: &Site, w: &Works, pal: &Palette) {
    let (p, yaw, s) = (site.origin, site.yaw, w.scale());
    let brick = *pal.brick.red;
    solid.block(p, 0.06 * s, 0.045, 0.03, yaw, brick, None, 0.03);
    solid.gable(p, 0.06 * s, 0.045, 0.03, 0.02, yaw, 0.003, *pal.slate.works, brick);
}
