use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::parts::gabled;
use crate::site::Site;

/// A handloom weaver: a thatched cottage, the same at any size.
pub fn hand_loom(solid: &mut MeshBuf, site: &Site, pal: &Palette) {
    gabled(solid, site.origin, 0.06, 0.04, 1.0, site.yaw, *pal.walls.weaver_cottage, *pal.thatch.weaver_cottage);
}
