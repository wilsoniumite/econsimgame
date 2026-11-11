//! Trees and bushes: the broadleaves and conifers of the woods, the smaller
//! trees of the hedgerows, and the gorse and broom of the heath.
//!
//! A tree has no age yet: the simulation's woods are a share of a cell, and
//! every tree slot in a wood is drawn full grown. `size` only varies them.

use glam::Vec3;

use crate::colour::{pick, shade};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Species {
    Broadleaf,
    Conifer,
}

/// A tree of the woods and fields.
#[derive(Clone, Copy, Debug)]
pub struct Tree {
    pub species: Species,
    /// 1 is a tree 0.11 units tall, which is more than a four-storey house; the woods run from 0.85 to 1.25
    pub size: f32,
    /// which of the species' greens it takes, from 0 to 1
    pub tint: f32,
    /// how its crown is turned, in radians, so that no two look alike
    pub turn: f32,
}

/// A tree: a trunk under a lumpy ball of leaves, or under two cones for a conifer.
pub fn tree(green: &mut MeshBuf, site: &Site, t: &Tree, pal: &Palette) {
    let (p, s) = (site.origin, t.size);
    let trunk = *pal.foliage.trunk;
    match t.species {
        Species::Conifer => {
            let g = pick(&pal.foliage.conifer, t.tint);
            green.frustum(p, 0.007 * s, 0.005 * s, 0.025 * s, 3, trunk, None);
            green.frustum(p + Vec3::Y * 0.02 * s, 0.048 * s, 0.0, 0.085 * s, 6, g, None);
            green.frustum(p + Vec3::Y * 0.062 * s, 0.035 * s, 0.0, 0.065 * s, 5, shade(g, 1.08), None);
        }
        Species::Broadleaf => {
            let g = pick(&pal.foliage.broadleaf, t.tint);
            green.frustum(p, 0.008 * s, 0.006 * s, 0.032 * s, 3, trunk, None);
            green.ball(p + Vec3::Y * 0.062 * s, 0.06 * s, 0.05 * s, 5, 3, g, t.turn);
        }
    }
}

/// A hedgerow tree: a smaller, plainer broadleaf, as there are many of them.
/// `tint` may run past 1; it goes round the greens again.
pub fn hedge_tree(green: &mut MeshBuf, site: &Site, size: f32, tint: f32, turn: f32, pal: &Palette) {
    let (p, s) = (site.origin, size);
    let g = pick(&pal.foliage.broadleaf, tint);
    green.frustum(p, 0.007 * s, 0.005 * s, 0.03 * s, 3, *pal.foliage.trunk, None);
    green.ball(p + Vec3::Y * 0.056 * s, 0.056 * s, 0.042 * s, 5, 3, g, turn);
}

/// A low bush on the heath: gorse for the lower half of `v`, broom for the upper.
pub fn bush(green: &mut MeshBuf, site: &Site, v: f32, turn: f32, pal: &Palette) {
    let col = if v < 0.5 { *pal.foliage.gorse } else { *pal.foliage.broom };
    green.ball(site.origin + Vec3::Y * 0.012, 0.022 + 0.01 * v, 0.014, 5, 2, col, turn);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(species: Species, size: f32) -> MeshBuf {
        let mut b = MeshBuf::default();
        tree(&mut b, &Site::level(Vec3::ZERO, 0.0), &Tree { species, size, tint: 0.3, turn: 1.0 }, &Palette::standard());
        assert!(b.is_sound());
        b
    }

    #[test]
    fn a_tree_scales_with_its_size() {
        for species in [Species::Broadleaf, Species::Conifer] {
            let (small, big) = (build(species, 0.6), build(species, 1.2));
            let (h0, h1) = (small.bounds().unwrap().1.y, big.bounds().unwrap().1.y);
            assert!((h1 / h0 - 2.0).abs() < 0.01, "{species:?}: {h0} and {h1}");
        }
    }

    #[test]
    fn a_conifer_is_taller_and_narrower_than_a_broadleaf() {
        let (b, c) = (build(Species::Broadleaf, 1.0).bounds().unwrap(), build(Species::Conifer, 1.0).bounds().unwrap());
        assert!(c.1.y > b.1.y);
        assert!(c.1.x - c.0.x < b.1.x - b.0.x);
    }

    #[test]
    fn a_hedge_tree_is_smaller_than_a_wood_s_and_a_bush_lower_still() {
        let pal = Palette::standard();
        let site = Site::level(Vec3::ZERO, 0.0);
        let (mut h, mut g) = (MeshBuf::default(), MeshBuf::default());
        hedge_tree(&mut h, &site, 1.0, 0.3, 1.0, &pal);
        bush(&mut g, &site, 0.2, 1.0, &pal);
        let top = |b: &MeshBuf| b.bounds().unwrap().1.y;
        assert!(top(&h) < top(&build(Species::Broadleaf, 1.0)));
        assert!(top(&g) < top(&h) * 0.5);
        // gorse and broom are different greens
        let mut broom = MeshBuf::default();
        bush(&mut broom, &site, 0.8, 1.0, &pal);
        assert_ne!(g.col[0], broom.col[0]);
    }
}
