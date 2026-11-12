use glam::Vec3;

use crate::colour::shade;
use crate::houses::roof::Cover;
use crate::kit::{Detail, End, Kit, Open, Opening, Over, Range, RoofSpec, Side, Wall, FOOT};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::scale::STOREY;
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
    /// how much of it to draw
    pub detail: Detail,
}

/// A pit: its spoil heap, and at a coal or iron mine a timber headframe with
/// its winding wheel and a small engine house. From far off the heap, the
/// legs of the headframe and the engine house's shell; at the distance the
/// game is played from the house has its roof, its door and window and its
/// stack; close to the headframe is braced, its wheel has a rim, and the
/// heap has a shoulder where the last tubs were tipped.
pub fn pit(solid: &mut MeshBuf, site: &Site, pit: &Pit, pal: &Palette) {
    let p = site.origin;
    let q = site.turn();
    let (far, near) = (pit.detail == Detail::Far, pit.detail == Detail::Near);
    let heap = match pit.kind {
        PitKind::Coal => *pal.ground.spoil_coal,
        PitKind::Ironstone => *pal.ground.spoil_ironstone,
        PitKind::Stone => *pal.ground.spoil_stone,
        PitKind::Clay => *pal.ground.spoil_clay,
    };
    let (r, h) = (0.05 + 0.02 * pit.v, 0.045 + 0.02 * pit.v);
    solid.frustum(p + q * Vec3::new(0.04, 0.0, 0.0), r, 0.006, h, if near { 9 } else { 7 }, heap, None);
    if near {
        solid.frustum_turned(p + q * Vec3::new(0.04 + r * 0.5, 0.0, r * 0.45), r * 0.42, 0.004, h * 0.5, 6, 0.6, 0.02, shade(heap, 0.94), None);
    }
    if pit.kind != PitKind::Coal && pit.kind != PitKind::Ironstone {
        return;
    }
    // a headframe: two legs and a stay, and the wheel the rope runs over
    let wood = *pal.timber.headframe;
    let iron = *pal.iron.winding_gear;
    let top = p + Vec3::Y * 0.085;
    let t = if near { 0.0034 } else { 0.006 };
    let feet = [Vec3::new(-0.015, 0.0, -0.012), Vec3::new(-0.015, 0.0, 0.012), Vec3::new(0.03, 0.0, 0.0)].map(|f| p + q * f);
    for f in feet {
        solid.beam(f, top, t, wood);
    }
    if !far {
        solid.block(top - Vec3::Y * 0.006, 0.012, 0.012, 0.012, site.yaw, iron, Some(iron), 0.0);
    }
    if near {
        // the braces between the legs, and the rim of the wheel standing over the housing
        for k in [0.35, 0.68] {
            solid.beam(feet[0].lerp(top, k), feet[1].lerp(top, k), 0.0018, shade(wood, 0.9));
            solid.beam(feet[1].lerp(top, k), feet[2].lerp(top, k), 0.0018, shade(wood, 0.9));
        }
        let hub = top + Vec3::Y * 0.01;
        const N: usize = 8;
        let rim = |i: usize, z: f32| {
            let a = i as f32 / N as f32 * std::f32::consts::TAU;
            hub + q * Vec3::new(a.cos() * 0.008, a.sin() * 0.008, z)
        };
        for i in 0..N {
            solid.quad(rim(i, 0.001), rim(i, -0.001), rim(i + 1, -0.001), rim(i + 1, 0.001), shade(iron, 0.9));
            solid.tri(hub + q * Vec3::new(0.0, 0.0, 0.001), rim(i, 0.001), rim(i + 1, 0.001), shade(iron, if i % 2 == 0 { 1.1 } else { 0.8 }));
            solid.tri(hub + q * Vec3::new(0.0, 0.0, -0.001), rim(i + 1, -0.001), rim(i, -0.001), shade(iron, if i % 2 == 0 { 0.9 } else { 0.7 }));
        }
    }
    // the winding engine's house, its door toward the shaft
    let mut k = Kit::new(solid, site, pal, pit.detail, 0.0);
    let brick = *pal.brick.red;
    let wall = Wall { col: brick, plinth: Some(shade(brick, 0.84)), frame: *pal.walls.window_frame, sill: *pal.stone.sill };
    let rg = Range::rect(-0.055, -0.025, -0.0125, 0.0125);
    let lie = Cover::Slate.lie();
    let glazing = Open::Paned { cols: 2, rows: 3, round: false, lit: false };
    let front = [Opening { u: 0.009, y: FOOT + 0.0058, w: 0.0066, h: 0.0086, kind: glazing }, Opening { u: 0.021, y: FOOT + 0.0058, w: 0.0066, h: 0.0086, kind: glazing }];
    let end = [Opening { u: 0.0125, y: FOOT, w: 0.0052, h: 0.0124, kind: Open::Door { paint: *pal.timber.door, over: Over::Nothing } }];
    let back = [Opening { u: 0.015, y: FOOT + 0.0058, w: 0.0066, h: 0.0086, kind: glazing }];
    let y0 = -k.drop(&rg);
    k.walls(&rg, y0, STOREY, &wall, [Side::free(&front), Side::free(&end), Side::free(&back), Side::PLAIN], true, true);
    k.roof(&rg, &RoofSpec { h: STOREY, rise: 0.0125 * 0.6, eaves: lie.eaves, verge: lie.verge, ends: [End::Gable, End::Gable], col: *pal.slate.works, gable: brick, thick: lie.thick });
    k.stack(-0.05, -0.006, 0.0052, 0.0052, STOREY, STOREY + 0.016, *pal.brick.chimney, 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mines_have_headframes_and_quarries_only_heaps() {
        let pal = Palette::standard();
        let build = |kind, detail| {
            let mut b = MeshBuf::default();
            pit(&mut b, &Site::level(Vec3::ZERO, 0.0), &Pit { kind, v: 0.5, detail }, &pal);
            assert!(b.is_sound());
            b
        };
        for detail in Detail::ALL {
            // a quarry or a clay pit is a heap and nothing else: from far off, seven sloping faces
            assert_eq!(build(PitKind::Stone, detail).triangles(), build(PitKind::Clay, detail).triangles());
            assert!(build(PitKind::Coal, detail).triangles() > build(PitKind::Stone, detail).triangles());
            assert!(build(PitKind::Coal, detail).bounds().unwrap().1.y > build(PitKind::Stone, detail).bounds().unwrap().1.y);
            assert_eq!(build(PitKind::Coal, detail).triangles(), build(PitKind::Ironstone, detail).triangles());
        }
        assert_eq!(build(PitKind::Stone, Detail::Far).triangles(), 14);
    }
}
