use super::{Kind, Works};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::parts::smoking_chimney;
use crate::site::Site;

/// A smithy (in stone) or a millwright's shop (in boards): a low shed under
/// slate with a smoking chimney through the roof.
pub fn workshop(solid: &mut MeshBuf, smoke: &mut MeshBuf, site: &Site, w: &Works, pal: &Palette) {
    let (p, yaw, s) = (site.origin, site.yaw, w.scale());
    let stone = *pal.stone.works;
    solid.block(p, 0.06 * s, 0.045 * s, 0.026, yaw, if w.kind == Kind::Smithy { stone } else { *pal.timber.boards }, None, 0.03);
    solid.gable(p, 0.06 * s, 0.045 * s, 0.026, 0.022, yaw, 0.004, *pal.slate.works, stone);
    smoking_chimney(solid, smoke, pal, site.at(0.02 * s, 0.0), 0.06, 0.006, w.seed);
}
