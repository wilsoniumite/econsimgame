//! What several generators share: a plain gabled building, and the height of
//! a storey of the buildings that are not houses or works. (A house's parts
//! are in `houses/kit.rs`, a works' in `works/kit.rs`.)

use glam::Vec3;

use crate::colour::Rgba;
use crate::mesh::MeshBuf;

/// The height of one storey of a barn, a church or a station, in map units
/// (the relief and the buildings are both drawn about three times life
/// size). A house's storey is lower, a works' between the two.
pub const STOREY: f32 = 0.024;

/// A plain building: four walls and a steep pitched roof, the ridge along its width.
#[allow(clippy::too_many_arguments)]
pub fn gabled(b: &mut MeshBuf, p: Vec3, w: f32, d: f32, storeys: f32, yaw: f32, wall: Rgba, roof: Rgba) {
    let h = STOREY * storeys;
    b.block(p, w, d, h, yaw, wall, None, 0.03);
    b.gable(p, w, d, h, d * 0.55, yaw, 0.004, roof, wall);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_gabled_building_is_as_tall_as_its_storeys_and_its_roof() {
        let mut b = MeshBuf::default();
        gabled(&mut b, Vec3::ZERO, 0.08, 0.036, 1.5, 0.0, [0.8, 0.7, 0.6, 1.0], [0.6, 0.3, 0.2, 1.0]);
        assert!(b.is_sound());
        let (lo, hi) = b.bounds().unwrap();
        // its walls reach below the ground for a slope, and its ridge stands above its eaves by its pitch
        assert!(lo.y < 0.0 && (hi.y - (STOREY * 1.5 + 0.036 * 0.55)).abs() < 1e-6, "from {} to {}", lo.y, hi.y);
    }
}
