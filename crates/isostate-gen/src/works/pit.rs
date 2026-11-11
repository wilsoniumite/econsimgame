use glam::Vec3;

use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// What is dug at a pit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PitKind {
    Coal,
    Ironstone,
    Stone,
    Clay,
}

/// A pit to draw.
#[derive(Clone, Copy, Debug)]
pub struct Pit {
    pub kind: PitKind,
    /// a number of the pit's own, from 0 to 1, which sets the size of its heap
    pub v: f32,
}

/// A pit: its spoil heap, and at a coal or iron mine a timber headframe with
/// its winding gear and a small engine house.
pub fn pit(solid: &mut MeshBuf, site: &Site, pit: &Pit, pal: &Palette) {
    let p = site.origin;
    let q = site.turn();
    let heap = match pit.kind {
        PitKind::Coal => *pal.ground.spoil_coal,
        PitKind::Ironstone => *pal.ground.spoil_ironstone,
        PitKind::Stone => *pal.ground.spoil_stone,
        PitKind::Clay => *pal.ground.spoil_clay,
    };
    solid.frustum(p + q * Vec3::new(0.04, 0.0, 0.0), 0.05 + 0.02 * pit.v, 0.006, 0.045 + 0.02 * pit.v, 7, heap, None);
    if pit.kind == PitKind::Coal || pit.kind == PitKind::Ironstone {
        // a headframe: two legs, a stay and a wheel housing
        let wood = *pal.timber.headframe;
        let top = p + Vec3::Y * 0.085;
        solid.beam(p + q * Vec3::new(-0.015, 0.0, -0.012), top, 0.006, wood);
        solid.beam(p + q * Vec3::new(-0.015, 0.0, 0.012), top, 0.006, wood);
        solid.beam(p + q * Vec3::new(0.03, 0.0, 0.0), top, 0.006, wood);
        solid.block(top - Vec3::Y * 0.006, 0.012, 0.012, 0.012, site.yaw, *pal.iron.winding_gear, Some(*pal.iron.winding_gear), 0.0);
        solid.block(p + q * Vec3::new(-0.04, 0.0, 0.0), 0.03, 0.025, 0.02, site.yaw, *pal.brick.red, Some(*pal.slate.works), 0.02);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mines_have_headframes_and_quarries_only_heaps() {
        let pal = Palette::standard();
        let build = |kind| {
            let mut b = MeshBuf::default();
            pit(&mut b, &Site::level(Vec3::ZERO, 0.0), &Pit { kind, v: 0.5 }, &pal);
            assert!(b.is_sound());
            b
        };
        // a heap is seven sloping faces and nothing else
        assert_eq!(build(PitKind::Stone).triangles(), 14);
        assert_eq!(build(PitKind::Clay).triangles(), 14);
        assert!(build(PitKind::Coal).triangles() > 14);
        assert!(build(PitKind::Coal).bounds().unwrap().1.y > build(PitKind::Stone).bounds().unwrap().1.y);
        assert_eq!(build(PitKind::Coal).triangles(), build(PitKind::Ironstone).triangles());
    }
}
