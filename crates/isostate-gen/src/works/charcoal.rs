use super::Works;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// Charcoal burning: a staggered line of turf-covered clamps, one more for each tier.
pub fn charcoal_burning(solid: &mut MeshBuf, site: &Site, w: &Works, pal: &Palette) {
    let tier = w.tier as f32;
    for i in 0..(tier as usize + 1) {
        let o = site.at((i as f32 - tier * 0.5) * 0.035, (i % 2) as f32 * 0.02);
        solid.frustum(o, 0.022, 0.008, 0.02, 7, *pal.ground.charcoal_clamp, Some(*pal.ground.charcoal_clamp_top));
    }
}
