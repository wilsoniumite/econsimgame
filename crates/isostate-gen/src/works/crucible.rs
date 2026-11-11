use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// A crucible steel shop: a stone shed with a squat conical stack beside
/// it, the same at any size.
pub fn crucible_steel(solid: &mut MeshBuf, site: &Site, pal: &Palette) {
    let (p, yaw) = (site.origin, site.yaw);
    let stone = *pal.stone.works;
    solid.block(p, 0.07, 0.045, 0.03, yaw, stone, None, 0.03);
    solid.gable(p, 0.07, 0.045, 0.03, 0.02, yaw, 0.003, *pal.slate.works, stone);
    solid.frustum(site.at(0.05, 0.0), 0.016, 0.006, 0.07, 6, *pal.brick.red, Some(*pal.smoke.soot));
}
