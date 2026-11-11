//! Farmsteads among the fields.

use glam::Vec3;

use crate::colour::pick;
use crate::hash::Seed;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::parts::gabled;
use crate::site::Site;
use crate::trees::{hedge_tree, Greenery};

/// A farmstead to draw. The site is the farmhouse; the yard and the barn lie behind it (toward -z).
#[derive(Clone, Copy, Debug)]
pub struct Farmstead {
    /// it grows corn (and so has a rick) rather than keeping stock on grass
    pub arable: bool,
    /// it stands in the hills: stone and slate in place of wash, brick, tile and thatch
    pub upland: bool,
    pub seed: Seed,
}

/// A farmstead: the house with its back to the hedge, a yard in front, the
/// barn across the yard and a cart shed or byre on one side; ricks of hay or
/// corn by an arable farm, and a tree or two. Stone and slate in the hills,
/// brick, tile and thatch in the lowland.
pub fn farmstead(solid: &mut MeshBuf, green: &mut Greenery, site: &Site, f: &Farmstead, pal: &Palette) {
    let Seed { u, v } = f.seed;
    let yaw = site.yaw;
    let wall = if f.upland { pick(&pal.stone.upland_farm, v) } else { pal.walls.farmhouse(v) };
    let roof = if f.upland { *pal.slate.upland_farm } else if v < 0.45 { *pal.thatch.farmhouse } else { *pal.roofs.tile_farm };
    let barn_wall = if f.upland { *pal.stone.upland_barn } else if u < 0.2 { *pal.timber.barn_tarred } else { *pal.brick.barn };
    let barn_roof = if f.upland { *pal.slate.upland_barn } else { *pal.roofs.barn };
    // the yard: beaten earth between the buildings
    let yc = site.at(0.0, -0.045);
    solid.block(yc - Vec3::Y * 0.004, 0.085, 0.055, 0.0055, yaw, *pal.ground.farmyard_edge, Some(*pal.ground.farmyard), 0.01);
    let storeys = 1.0 + (v > 0.55) as u8 as f32;
    gabled(solid, site.at(0.0, 0.0), 0.05, 0.032, storeys, yaw, wall, roof);
    // the barn across the yard, long and tall
    gabled(solid, site.at(-0.005, -0.09), 0.08, 0.036, 1.3, yaw, barn_wall, barn_roof);
    // a cart shed or byre closing one side of the bigger yards
    let side = if u < 0.16 { 1.0 } else { -1.0 };
    if v > 0.55 {
        gabled(solid, site.at(0.052 * side, -0.046), 0.05, 0.026, 0.85, yaw + std::f32::consts::FRAC_PI_2, barn_wall, barn_roof);
    }
    if f.arable {
        // a rick of corn or hay
        solid.frustum(site.at(-0.06 * side, -0.03), 0.013, 0.0, 0.024, 5, *pal.thatch.rick, None);
    }
    // a tree by the house
    if v < 0.6 {
        hedge_tree(green, &site.moved(site.at(0.05 * -side, 0.03), 0.0), 0.7 + 0.2 * u, v, yaw, pal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site::Model;

    fn build(arable: bool, upland: bool, seed: Seed) -> Model {
        let mut m = Model::default();
        farmstead(&mut m.solid, &mut m.green, &Site::level(Vec3::ZERO, 0.0), &Farmstead { arable, upland, seed }, &Palette::standard());
        assert!(m.is_sound());
        m
    }

    #[test]
    fn the_yard_and_the_barn_lie_behind_the_house() {
        for n in 0..30 {
            let m = build(true, false, Seed::of(n));
            let (lo, hi) = m.solid.bounds().unwrap();
            // the house is at the origin; everything else is toward -z
            assert!(lo.z < -0.1 && hi.z < 0.03, "seed {n}: from {} to {}", lo.z, hi.z);
            assert!(hi.x - lo.x < 0.2);
        }
    }

    #[test]
    fn hill_farms_are_stone_and_slate() {
        let pal = Palette::standard();
        let seed = Seed::new(0.5, 0.3);
        let (low, high) = (build(false, false, seed), build(false, true, seed));
        assert!(high.solid.col.contains(&*pal.slate.upland_farm));
        assert!(!low.solid.col.contains(&*pal.slate.upland_farm));
        assert!(low.solid.col.contains(&*pal.thatch.farmhouse));
    }

    #[test]
    fn an_arable_farm_has_a_rick() {
        let seed = Seed::new(0.5, 0.3);
        assert_eq!(build(true, false, seed).solid.triangles(), build(false, false, seed).solid.triangles() + 5);
    }

    #[test]
    fn the_bigger_farms_have_a_second_storey_and_a_cart_shed_and_the_smaller_a_tree() {
        let (small, big) = (build(false, false, Seed::new(0.5, 0.3)), build(false, false, Seed::new(0.5, 0.7)));
        assert!(big.solid.triangles() > small.solid.triangles());
        assert!(!small.green.leaves.is_empty() && !small.green.bare.is_empty());
        assert!(big.green.is_empty());
    }
}
