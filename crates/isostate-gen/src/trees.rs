//! Trees and bushes: the broadleaves and conifers of the woods, the smaller
//! trees of the hedgerows, and the gorse and broom of the heath.
//!
//! A tree has no age yet: the simulation's woods are a share of a cell, and
//! every tree slot in a wood is drawn full grown. `size` only varies them.
//!
//! A broadleaf tree has two forms, and both are built every time: its crown
//! of leaves, and the bare limbs it shows when they are down. They go into
//! different buffers of a `Greenery`, so that the viewer can turn the leaves
//! through the year and change one form for the other by showing and hiding,
//! without building anything again (`in_leaf` says which is to be seen).
//! Conifers, gorse and broom keep their green, and stay as they are.

use glam::Vec3;

use crate::colour::{pick, shade, Rgba};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// What trees and bushes are built into: three buffers, as the viewer draws
/// each its own way through the year.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Greenery {
    /// trunks, conifers and bushes: the same all year
    pub evergreen: MeshBuf,
    /// the crowns of the broadleaf trees, which turn with the seasons and
    /// are not drawn while the leaves are down
    pub leaves: MeshBuf,
    /// the bare limbs of the broadleaf trees, drawn only while the leaves
    /// are down. They are flat blades seen from both sides.
    pub bare: MeshBuf,
}

impl Greenery {
    pub fn parts(&self) -> [&MeshBuf; 3] {
        [&self.evergreen, &self.leaves, &self.bare]
    }

    pub fn parts_mut(&mut self) -> [&mut MeshBuf; 3] {
        [&mut self.evergreen, &mut self.leaves, &mut self.bare]
    }

    pub fn is_empty(&self) -> bool {
        self.parts().into_iter().all(|b| b.is_empty())
    }
}

/// The island's broadleaf trees are in this many groups, each coming into
/// leaf and shedding at a time of its own, so that a wood turns over some
/// weeks and not in a day.
pub const LEAF_GROUPS: usize = 4;

/// The group a tree is in, by a number of its own from 0 to 1.
pub fn leaf_group(v: f32) -> usize {
    ((v * 53.7).fract() * LEAF_GROUPS as f32) as usize % LEAF_GROUPS
}

/// Whether the trees of a group are in leaf at a phase of the year (0
/// midwinter, 0.25 spring, 0.5 midsummer, 0.75 autumn): the buds break from
/// the middle of March and the last leaves are down by the end of November.
pub fn in_leaf(phase: f32, group: usize) -> bool {
    let late = 0.06 * (group % LEAF_GROUPS) as f32 / LEAF_GROUPS as f32;
    let phase = phase.rem_euclid(1.0);
    phase >= 0.19 + late && phase < 0.84 + late
}

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

/// A tree: a trunk under a lumpy ball of leaves (and the bare limbs that
/// show when they are down), or under two cones for a conifer.
pub fn tree(green: &mut Greenery, site: &Site, t: &Tree, pal: &Palette) {
    let (p, s) = (site.origin, t.size);
    let trunk = *pal.foliage.trunk;
    match t.species {
        Species::Conifer => {
            let g = pick(&pal.foliage.conifer, t.tint);
            let b = &mut green.evergreen;
            b.frustum(p, 0.007 * s, 0.005 * s, 0.025 * s, 3, trunk, None);
            b.frustum(p + Vec3::Y * 0.02 * s, 0.048 * s, 0.0, 0.085 * s, 6, g, None);
            b.frustum(p + Vec3::Y * 0.062 * s, 0.035 * s, 0.0, 0.065 * s, 5, shade(g, 1.08), None);
        }
        Species::Broadleaf => {
            let g = pick(&pal.foliage.broadleaf, t.tint);
            // the trunk runs up through the crown to a point, as the leader of the bare tree
            green.evergreen.frustum(p, 0.009 * s, 0.0, 0.104 * s, 3, trunk, None);
            green.leaves.ball(p + Vec3::Y * 0.062 * s, 0.06 * s, 0.05 * s, 5, 3, g, t.turn);
            limbs(&mut green.bare, p, s, t.turn, *pal.foliage.bare_limb);
        }
    }
}

/// A hedgerow tree: a smaller, plainer broadleaf, as there are many of them.
/// `tint` may run past 1; it goes round the greens again.
pub fn hedge_tree(green: &mut Greenery, site: &Site, size: f32, tint: f32, turn: f32, pal: &Palette) {
    let (p, s) = (site.origin, size);
    let g = pick(&pal.foliage.broadleaf, tint);
    green.evergreen.frustum(p, 0.007 * s, 0.0, 0.088 * s, 3, *pal.foliage.trunk, None);
    green.leaves.ball(p + Vec3::Y * 0.056 * s, 0.056 * s, 0.042 * s, 5, 3, g, turn);
    limbs(&mut green.bare, p, s * 0.9, turn, *pal.foliage.bare_limb);
}

/// A low bush on the heath: gorse for the lower half of `v`, broom for the upper.
pub fn bush(green: &mut Greenery, site: &Site, v: f32, turn: f32, pal: &Palette) {
    let col = if v < 0.5 { *pal.foliage.gorse } else { *pal.foliage.broom };
    green.evergreen.ball(site.origin + Vec3::Y * 0.012, 0.022 + 0.01 * v, 0.014, 5, 2, col, turn);
}

/// How many limbs a bare tree shows.
const LIMBS: usize = 7;

/// The limbs of a bare broadleaf tree of size `s` standing at `p`: flat
/// tapered blades that leave the trunk at different heights and turns, the
/// lower ones longer and nearer level, so that they cross as they are seen.
/// A blade is one triangle, drawn from both sides.
fn limbs(bare: &mut MeshBuf, p: Vec3, s: f32, turn: f32, col: Rgba) {
    // there are tens of thousands of trees, so the limbs of a tree of size 1 are worked out
    // once, in a few shapes, and each tree turns and scales one of them
    static SHAPES: std::sync::OnceLock<[[Limb; LIMBS]; LIMB_SHAPES]> = std::sync::OnceLock::new();
    let shapes = SHAPES.get_or_init(|| std::array::from_fn(|shape| std::array::from_fn(|i| limb(shape, i))));
    let (sin, cos) = turn.sin_cos();
    // the same turn about the upright as a site's yaw
    let place = |v: Vec3| p + Vec3::new(v.x * cos + v.z * sin, v.y, -v.x * sin + v.z * cos) * s;
    for l in &shapes[(turn.abs() * 5.0) as usize % LIMB_SHAPES] {
        bare.tri(place(l.corners[0]), place(l.corners[1]), place(l.corners[2]), shade(col, l.shade));
    }
}

/// How many shapes of bare tree there are.
const LIMB_SHAPES: usize = 4;

/// One limb of a bare tree of size 1 standing at the origin: the two corners
/// of the blade at the trunk and its tip, and how light it is.
#[derive(Clone, Copy)]
struct Limb {
    corners: [Vec3; 3],
    shade: f32,
}

/// The `i`th limb up the trunk of a bare tree of the given shape.
fn limb(shape: usize, i: usize) -> Limb {
    let f = i as f32 / (LIMBS - 1) as f32;
    // round the trunk by the golden angle, so no two limbs lie over one another
    let a = i as f32 * 2.399 + shape as f32 * 0.9;
    let rise = 0.55 + 0.5 * f + 0.12 * (shape as f32 * 2.3 + i as f32).sin();
    let from = Vec3::Y * (0.042 + 0.043 * f);
    let dir = Vec3::new(rise.cos() * a.cos(), rise.sin(), rise.cos() * a.sin());
    let len = 0.052 - 0.02 * f;
    // the blade's width lies half across and half up, so it is seen from above and from the side
    let across = Vec3::new(-a.sin(), 0.0, a.cos());
    let width = (across + dir.cross(across) * 0.7).normalize_or_zero() * (0.0085 - 0.003 * f);
    Limb { corners: [from - width, from + width, from + dir * len], shade: 0.9 + 0.2 * f }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(species: Species, size: f32) -> Greenery {
        let mut g = Greenery::default();
        tree(&mut g, &Site::level(Vec3::ZERO, 0.0), &Tree { species, size, tint: 0.3, turn: 1.0 }, &Palette::standard());
        assert!(g.parts().into_iter().all(|b| b.is_sound()));
        g
    }

    /// The top of the tree as it stands in summer, and in winter.
    fn tops(g: &Greenery) -> (f32, f32) {
        let top = |b: &MeshBuf| b.bounds().map_or(0.0, |(_, hi)| hi.y);
        (top(&g.evergreen).max(top(&g.leaves)), top(&g.evergreen).max(top(&g.bare)))
    }

    #[test]
    fn a_tree_scales_with_its_size() {
        for species in [Species::Broadleaf, Species::Conifer] {
            let (small, big) = (build(species, 0.6), build(species, 1.2));
            let (h0, h1) = (tops(&small).0, tops(&big).0);
            assert!((h1 / h0 - 2.0).abs() < 0.01, "{species:?}: {h0} and {h1}");
        }
    }

    #[test]
    fn a_conifer_is_taller_and_narrower_than_a_broadleaf() {
        let (b, c) = (build(Species::Broadleaf, 1.0), build(Species::Conifer, 1.0));
        assert!(tops(&c).0 > tops(&b).0);
        let width = |m: &MeshBuf| m.bounds().map(|(lo, hi)| hi.x - lo.x).unwrap();
        assert!(width(&c.evergreen) < width(&b.leaves));
        // a conifer keeps its green: nothing of it turns or falls
        assert!(c.leaves.is_empty() && c.bare.is_empty());
    }

    #[test]
    fn a_broadleaf_has_a_crown_and_the_bare_limbs_under_it() {
        let pal = Palette::standard();
        for n in 0..20 {
            let mut g = Greenery::default();
            let t = Tree { species: Species::Broadleaf, size: 0.85 + 0.02 * n as f32, tint: 0.05 * n as f32, turn: n as f32 * 0.7 };
            tree(&mut g, &Site::level(Vec3::new(2.0, 0.5, 3.0), 0.0), &t, &pal);
            // the trunk is wood, the crown is leaves, the limbs are a blade each
            assert!(g.evergreen.col.iter().all(|c| c[1] < c[0]) && g.leaves.col.iter().all(|c| c[1] > c[0]));
            assert_eq!(g.bare.triangles(), LIMBS);
            // in winter the tree is no taller or wider than in leaf (a crown is 0.06 across its widest), and stands on the same spot
            let reach = g.bare.pos.iter().map(|q| ((q[0] - 2.0).powi(2) + (q[2] - 3.0).powi(2)).sqrt()).fold(0.0f32, f32::max);
            assert!(reach < 0.06 * t.size, "tree {n}: a limb reaches {reach}");
            let (summer, winter) = tops(&g);
            assert!(winter <= summer + 0.004 && winter > summer * 0.8, "tree {n}: {winter} against {summer}");
            // the limbs reach out on more than one side of the trunk
            let limbs = g.bare.bounds().unwrap();
            assert!(limbs.1.x - limbs.0.x > 0.03 && limbs.1.z - limbs.0.z > 0.03 && limbs.0.x < 2.0 && limbs.1.x > 2.0);
            // and the bare tree is cheap: a third of the crown's corners
            assert!(g.bare.pos.len() * 2 < g.leaves.pos.len());
        }
    }

    #[test]
    fn the_leaves_are_out_from_spring_to_autumn_a_group_at_a_time() {
        for g in 0..LEAF_GROUPS {
            assert!(!in_leaf(0.0, g) && !in_leaf(0.1, g) && !in_leaf(0.95, g) && !in_leaf(1.0, g));
            assert!(in_leaf(0.25, g) && in_leaf(0.5, g) && in_leaf(0.75, g) && in_leaf(0.83, g));
            // the year goes round
            assert_eq!(in_leaf(1.5, g), in_leaf(0.5, g));
        }
        // the groups turn one after another
        let out_at = |g: usize| (0..1000).map(|i| i as f32 / 1000.0).find(|&p| in_leaf(p, g)).unwrap();
        let down_at = |g: usize| (500..1000).map(|i| i as f32 / 1000.0).find(|&p| !in_leaf(p, g)).unwrap();
        for g in 1..LEAF_GROUPS {
            assert!(out_at(g) > out_at(g - 1) && down_at(g) > down_at(g - 1));
        }
        // and trees fall into every group
        let mut seen = [0; LEAF_GROUPS];
        for i in 0..400 {
            seen[leaf_group(i as f32 / 400.0)] += 1;
        }
        assert!(seen.iter().all(|&n| n > 60), "{seen:?}");
    }

    #[test]
    fn a_hedge_tree_is_smaller_than_a_wood_s_and_a_bush_lower_still() {
        let pal = Palette::standard();
        let site = Site::level(Vec3::ZERO, 0.0);
        let (mut h, mut g) = (Greenery::default(), Greenery::default());
        hedge_tree(&mut h, &site, 1.0, 0.3, 1.0, &pal);
        bush(&mut g, &site, 0.2, 1.0, &pal);
        assert!(tops(&h).0 < tops(&build(Species::Broadleaf, 1.0)).0);
        assert!(tops(&g).0 < tops(&h).0 * 0.5);
        // it has its winter form too, and a bush has none
        assert_eq!(h.bare.triangles(), LIMBS);
        assert!(g.leaves.is_empty() && g.bare.is_empty());
        // gorse and broom are different greens
        let mut broom = Greenery::default();
        bush(&mut broom, &site, 0.8, 1.0, &pal);
        assert_ne!(g.evergreen.col[0], broom.evergreen.col[0]);
    }
}
