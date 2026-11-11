use glam::{Quat, Vec3};

use crate::hash::Seed;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::parts::smoking_chimney;
use crate::site::Site;

/// One of the sheds and warehouses that stand about a works yard.
#[derive(Clone, Copy, Debug)]
pub struct YardShed {
    pub seed: Seed,
    /// a number of the shed's own, which sets how its smoke drifts if it has a chimney
    pub smoke_seed: f32,
}

/// A yard shed: brick or stone under a low slate roof, its size and wall by
/// its seed, and one in eight or so with a chimney.
pub fn yard_shed(solid: &mut MeshBuf, smoke: &mut MeshBuf, site: &Site, y: &YardShed, pal: &Palette) {
    let (p, yaw) = (site.origin, site.yaw);
    let Seed { u, v } = y.seed;
    let wall = pal.yard_shed_wall(v);
    let (w, d, h) = (0.06 + 0.04 * v, 0.045 + 0.015 * u, 0.026 + 0.018 * v);
    solid.block(p, w, d, h, yaw, wall, None, 0.03);
    solid.gable(p, w, d, h, d * 0.35, yaw, 0.003, *pal.slate.works, wall);
    if u < 0.12 {
        let cp = p + Quat::from_rotation_y(yaw) * Vec3::new(w * 0.35, 0.0, 0.0);
        smoking_chimney(solid, smoke, pal, cp, h + 0.07, 0.006, y.smoke_seed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site::Model;

    #[test]
    fn sheds_come_in_sizes_and_a_few_have_chimneys() {
        let pal = Palette::standard();
        let mut smoking = 0;
        for n in 0..200 {
            let mut m = Model::default();
            yard_shed(&mut m.solid, &mut m.smoke, &Site::level(Vec3::ZERO, 0.0), &YardShed { seed: Seed::of(n), smoke_seed: 0.0 }, &pal);
            assert!(m.is_sound());
            let (lo, hi) = m.solid.bounds().unwrap();
            assert!(hi.x - lo.x <= 0.1 + 0.007 && hi.z - lo.z <= 0.06 + 0.007);
            smoking += !m.smoke.is_empty() as usize;
        }
        assert!((10..45).contains(&smoking), "{smoking} of 200 have a chimney");
    }
}
