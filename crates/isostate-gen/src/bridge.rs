//! Bridges that carry roads over rivers.
//!
//! A bridge is placed by its crossing, not by a `Site`: the point where the
//! road meets the middle of the river, the way the road runs, the river's
//! half-width there and its water level. The ground is asked for the height
//! of the bank at each end.

use glam::{Vec2, Vec3};

use crate::colour::shade;
use crate::mesh::MeshBuf;
use crate::palette::Palette;

/// A road's crossing of a river.
#[derive(Clone, Copy, Debug)]
pub struct RoadBridge {
    /// where the road meets the middle of the river, on the map (x, z)
    pub at: Vec2,
    /// the way the road runs, a unit vector on the map
    pub along: Vec2,
    /// half the river's width at the crossing
    pub half: f32,
    /// the height of the water
    pub level: f32,
    /// a main road, which is wider and built in paler stone, rather than a lane
    pub main: bool,
}

impl RoadBridge {
    /// How long the bridge is, bank to bank.
    pub fn span(&self) -> f32 {
        2.0 * self.half + 0.035
    }

    /// How many arches carry it: two over a wide river.
    pub fn arches(&self) -> usize {
        if self.span() > 0.11 { 2 } else { 1 }
    }
}

/// A stone bridge carrying a road over a river: a hump-backed deck with
/// parapets, on an arch (two for a wide river) whose dark opening shows on
/// either face. `ground` gives the height of the land at a map point (x, z).
pub fn road_bridge(solid: &mut MeshBuf, b: &RoadBridge, ground: &dyn Fn(f32, f32) -> f32, pal: &Palette) {
    let (half, level, along) = (b.half, b.level, b.along);
    let stone = if b.main { *pal.bridges.road_stone } else { *pal.bridges.lane_stone };
    let dark = *pal.bridges.arch_shadow;
    let span = 2.0 * half + 0.035;
    let width = if b.main { 0.042 } else { 0.03 };
    let dir = Vec3::new(along.x, 0.0, along.y);
    let side = Vec3::new(-along.y, 0.0, along.x);
    let centre = Vec3::new(b.at.x, 0.0, b.at.y);
    // the deck meets the road on each bank and rises over the water
    let road = |t: f32| {
        let p = centre + dir * (span * (t - 0.5));
        ground(p.x, p.z).max(level) + 0.012
    };
    let ends = road(0.0).max(road(1.0));
    let hump = 0.012 + 0.08 * half;
    let segs = 8;
    let deck = |t: f32| -> Vec3 {
        let p = centre + dir * (span * (t - 0.5));
        let y = ends.max(level + 0.02) + hump * (std::f32::consts::PI * t).sin();
        Vec3::new(p.x, y, p.z)
    };
    let arches = if span > 0.11 { 2 } else { 1 };
    for k in 0..segs {
        let (t0, t1) = (k as f32 / segs as f32, (k + 1) as f32 / segs as f32);
        let (p0, p1) = (deck(t0), deck(t1));
        // the roadway
        solid.quad(p0 + side * (width * 0.5), p1 + side * (width * 0.5), p1 - side * (width * 0.5), p0 - side * (width * 0.5), shade(stone, 1.05));
        for s in [-1.0f32, 1.0] {
            let (e0, e1) = (p0 + side * (width * 0.5 * s), p1 + side * (width * 0.5 * s));
            let base = level - 0.01;
            let (f0, f1) = (Vec3::new(e0.x, base, e0.z), Vec3::new(e1.x, base, e1.z));
            let col = if s > 0.0 { stone } else { shade(stone, 0.86) };
            // the face down to the water, and the parapet above the deck
            if s > 0.0 {
                solid.quad(f0, f1, e1, e0, col);
                solid.quad(e0, e1, e1 + Vec3::Y * 0.009, e0 + Vec3::Y * 0.009, shade(col, 1.04));
            } else {
                solid.quad(f1, f0, e0, e1, col);
                solid.quad(e1, e0, e0 + Vec3::Y * 0.009, e1 + Vec3::Y * 0.009, shade(col, 1.04));
            }
        }
    }
    // the arch openings, painted dark on both faces, just proud of the stone
    for a in 0..arches {
        let (a0, a1) = (a as f32 / arches as f32, (a + 1) as f32 / arches as f32);
        let mid = 0.5 * (a0 + a1);
        let half_t = 0.42 * (a1 - a0);
        let rise = (deck(mid).y - level - 0.012).max(0.006);
        for s in [-1.0f32, 1.0] {
            let off = side * ((width * 0.5 + 0.0012) * s);
            let pt = |u: f32| -> Vec3 {
                let t = mid + half_t * u.cos();
                let p = centre + dir * (span * (t - 0.5)) + off;
                Vec3::new(p.x, level - 0.004 + rise * u.sin(), p.z)
            };
            let foot = Vec3::new(0.0, level - 0.004, 0.0);
            for k in 0..6 {
                let (u0, u1) = (k as f32 / 6.0 * std::f32::consts::PI, (k + 1) as f32 / 6.0 * std::f32::consts::PI);
                let (q0, q1) = (pt(u0), pt(u1));
                let c = centre + dir * (span * (mid - 0.5)) + off + foot;
                if s > 0.0 {
                    solid.tri(c, q0, q1, dark);
                } else {
                    solid.tri(c, q1, q0, dark);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(half: f32, main: bool) -> MeshBuf {
        let mut m = MeshBuf::default();
        let b = RoadBridge { at: Vec2::ZERO, along: Vec2::X, half, level: 0.0, main };
        road_bridge(&mut m, &b, &|_, _| 0.01, &Palette::standard());
        assert!(m.is_sound());
        m
    }

    #[test]
    fn a_wide_river_gets_two_arches() {
        let (narrow, wide) = (build(0.02, false), build(0.06, false));
        // a deck of eight lengths, each with two faces and two parapets, and six dark wedges a face for each arch
        assert_eq!(narrow.triangles(), 8 * 10 + 12);
        assert_eq!(wide.triangles(), 8 * 10 + 24);
    }

    #[test]
    fn it_spans_the_river_and_humps_over_it() {
        let m = build(0.04, true);
        let (lo, hi) = m.bounds().unwrap();
        assert!((hi.x - lo.x - 0.115).abs() < 1e-5);
        // the crown is above the ends, which meet the road on the banks
        assert!(hi.y > 0.022 + 0.012);
        // a main road's bridge is wider than a lane's
        let lane = build(0.04, false).bounds().unwrap();
        assert!(hi.z - lo.z > lane.1.z - lane.0.z);
    }
}
