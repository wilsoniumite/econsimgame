//! Bridges that carry roads over rivers.
//!
//! A bridge is placed by its crossing, not by a `Site`: the point where the
//! road meets the middle of the river, the way the road runs, the river's
//! half-width there and its water level. The ground is asked for the height
//! of the bank at each end.

use glam::{Vec2, Vec3};

use crate::colour::{shade, Rgba};
use crate::kit::Detail;
use crate::mesh::MeshBuf;
use crate::palette::Palette;

/// A road's crossing of a river.
#[derive(Clone, Copy, Debug)]
pub struct RoadBridge {
    /// where the road meets the middle of the river, on the map (x, z)
    pub at: Vec2,
    /// the way the road runs, a unit vector on the map
    pub along: Vec2,
    /// half the river's width at the crossing, as the road crosses it: more
    /// than the river's own where the road runs aslant
    pub half: f32,
    /// the height of the water
    pub level: f32,
    /// a main road, which is wider and built in paler stone, rather than a lane
    pub main: bool,
    /// how much of it to draw: from far off the roadway on its arches, each
    /// in three flat lengths; at the distance the game is played from its
    /// parapets and cutwaters, the arches in six; close to a coping on the
    /// parapets and a string course under them
    pub detail: Detail,
}

impl RoadBridge {
    /// How long the bridge is, bank to bank.
    pub fn span(&self) -> f32 {
        2.0 * self.half + 0.035
    }

    /// How many arches carry it: two over a wide river, three over the widest.
    pub fn arches(&self) -> usize {
        if self.span() > 0.2 {
            3
        } else if self.span() > 0.11 {
            2
        } else {
            1
        }
    }

    /// How wide it is over its parapets: a little wider than its road.
    pub fn width(&self) -> f32 {
        if self.main { 0.042 } else { 0.03 }
    }

    /// Half the width of the roadway between the parapets' inner faces.
    pub fn inner_half(&self) -> f32 {
        self.width() * 0.5 - PARAPET.1
    }

    /// How far a ramp runs, as a share of the bridge's length bank to bank.
    pub fn ramp(&self) -> f32 {
        APPROACH / self.span()
    }

    /// The point of the map `t` of the way along the bridge's middle line: 0
    /// at one bank and 1 at the other, below 0 and above 1 on the ramps.
    pub fn point(&self, t: f32) -> Vec2 {
        self.at + self.along * (self.span() * (t - 0.5))
    }

    /// How far along the bridge (as for `point`) a point of the map lies,
    /// measured along its middle line.
    pub fn t_of(&self, p: Vec2) -> f32 {
        (p - self.at).dot(self.along) / self.span() + 0.5
    }

    /// The roadway as it is drawn, from the foot of one ramp to the foot of
    /// the other: the points where its flat lengths meet, in order, each as
    /// how far along the bridge it is (`point`) and the roadway's height
    /// there. Between two of them the roadway is flat from side to side and
    /// straight from one to the next. A cart on the bridge rides this.
    pub fn roadway(&self, ground: &dyn Fn(f32, f32) -> f32) -> Vec<(f32, f32)> {
        let deck = deck(self, ground);
        let mut ts: Vec<f32> = lengths(self, &deck).iter().flat_map(|l| [l.0, l.2]).collect();
        ts.sort_by(f32::total_cmp);
        ts.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
        ts.into_iter().map(|t| (t, deck(t))).collect()
    }
}

/// The height of the roadway at `t` of the way along the bridge (as for
/// `RoadBridge::point`): it meets the road on each bank, rises over the
/// water in a hump, and comes down a ramp at each end to just under the
/// road's own surface.
fn deck<'a>(b: &'a RoadBridge, ground: &'a dyn Fn(f32, f32) -> f32) -> impl Fn(f32) -> f32 + 'a {
    let (half, level, along) = (b.half, b.level, b.along);
    let span = b.span();
    let dir = Vec3::new(along.x, 0.0, along.y);
    let centre = Vec3::new(b.at.x, 0.0, b.at.y);
    // the roadway meets the road on each bank and rises over the water
    let road = move |t: f32| {
        let p = centre + dir * (span * (t - 0.5));
        ground(p.x, p.z).max(level) + 0.012
    };
    let ends = (road(0.0).max(level + 0.02), road(1.0).max(level + 0.02));
    let hump = 0.007 + 0.09 * half;
    move |t: f32| {
        if t < 0.0 {
            // the foot of a ramp, just under the road's own surface
            (road(t) - 0.006).min(ends.0)
        } else if t > 1.0 {
            (road(t) - 0.006).min(ends.1)
        } else {
            ends.0 + (ends.1 - ends.0) * t + hump * (std::f32::consts::PI * t).sin()
        }
    }
}

/// How high an arch rises over the water from its springing, given the roadway.
fn rise(b: &RoadBridge, deck: &dyn Fn(f32) -> f32, a: usize) -> f32 {
    let bay = 1.0 / b.arches() as f32;
    let spring = b.level - 0.003;
    (deck((a as f32 + 0.5) * bay) - 0.009 - spring).max(0.006)
}

/// The bridge in lengths along it, each with the height of its underside at
/// either end and whether it is the underside of an arch: a short ramp on
/// each bank, then for each arch an abutment or half a pier, the arch in flat
/// lengths, and half a pier or an abutment. The roadway is drawn over the
/// same lengths, so these are where it bends.
fn lengths(b: &RoadBridge, deck: &dyn Fn(f32) -> f32) -> Vec<(f32, f32, f32, f32, bool)> {
    // the stone goes down below the water, and the arches spring from just under its surface
    let (foot, spring) = (b.level - 0.01, b.level - 0.003);
    let arches = b.arches();
    let arc = if b.detail == Detail::Far { ARC_FAR } else { ARC };
    let bay = 1.0 / arches as f32;
    let reach = 0.4 * bay;
    let mut lengths: Vec<(f32, f32, f32, f32, bool)> = Vec::new();
    // a short ramp on each bank brings the roadway down to the road
    let ramp = b.ramp();
    lengths.push((-ramp, foot, 0.0, foot, false));
    lengths.push((1.0, foot, 1.0 + ramp, foot, false));
    for a in 0..arches {
        let mid = (a as f32 + 0.5) * bay;
        lengths.push((a as f32 * bay, foot, mid - reach, foot, false));
        for k in 0..arc {
            let (u0, u1) = (k as f32 / arc as f32 * std::f32::consts::PI, (k + 1) as f32 / arc as f32 * std::f32::consts::PI);
            lengths.push((mid - reach * u0.cos(), spring + rise(b, deck, a) * u0.sin(), mid - reach * u1.cos(), spring + rise(b, deck, a) * u1.sin(), true));
        }
        lengths.push((mid + reach, foot, (a + 1) as f32 * bay, foot, false));
    }
    lengths
}

/// How high a parapet stands above the roadway, and how thick it is.
const PARAPET: (f32, f32) = (0.007, 0.0035);
/// How many flat lengths an arch is drawn in: from the play distance and nearer, and from far off.
const ARC: usize = 6;
const ARC_FAR: usize = 3;
/// How far the ramp at each end runs back along the road from the bank.
pub const APPROACH: f32 = 0.02;

/// A stone bridge carrying a road over a river: a hump-backed roadway
/// between parapets, on a round arch (two or three over a wide river) that
/// is open from face to face, so the water shows through it; where there is
/// more than one arch the piers between them have cutwaters. `ground` gives
/// the height of the land at a map point (x, z).
pub fn road_bridge(solid: &mut MeshBuf, b: &RoadBridge, ground: &dyn Fn(f32, f32) -> f32, pal: &Palette) {
    let (level, along) = (b.level, b.along);
    let stone = if b.main { *pal.bridges.road_stone } else { *pal.bridges.lane_stone };
    let under = *pal.bridges.arch_shadow;
    let roadway = if b.main { *pal.ground.road_gravel } else { *pal.ground.lane_dust };
    let span = b.span();
    let dir = Vec3::new(along.x, 0.0, along.y);
    let side = Vec3::new(-along.y, 0.0, along.x);
    let centre = Vec3::new(b.at.x, 0.0, b.at.y);
    // a point of the bridge: `t` of the way along it, `out` to one side of its middle line, at a height
    let at = |t: f32, out: f32, y: f32| {
        let p = centre + dir * (span * (t - 0.5)) + side * out;
        Vec3::new(p.x, y, p.z)
    };
    // the roadway's height along it (`deck`, which a cart on the bridge reads too)
    let deck = deck(b, ground);
    // the stone goes down below the water, and the arches spring from just under its surface
    let (foot, spring) = (level - 0.01, level - 0.003);

    // the bridge in lengths along it, each with the height of its underside at either end (`lengths`)
    let arches = b.arches();
    let (far, near) = (b.detail == Detail::Far, b.detail == Detail::Near);
    let bay = 1.0 / arches as f32;
    let rise = |a: usize| rise(b, &deck, a);
    let lengths = lengths(b, &deck);
    let ramp = b.ramp();

    let (edge, inner) = (b.width() * 0.5, b.inner_half());
    for &(t0, low0, t1, low1, arch) in &lengths {
        let (d0, d1) = (deck(t0), deck(t1));
        // the parapets run down to nothing at the foot of each ramp
        let parapet = |t: f32| if (0.0..=1.0).contains(&t) && !far { PARAPET.0 } else { 0.0 };
        let (top0, top1) = (d0 + parapet(t0), d1 + parapet(t1));
        // the roadway, in the road's own gravel or dust
        solid.quad(at(t0, inner, d0), at(t1, inner, d1), at(t1, -inner, d1), at(t0, -inner, d0), roadway);
        // each face from the underside to the top of the parapet, then the parapet's top and its inner side
        let lit = shade(stone, 1.0);
        let shaded = shade(stone, 0.94);
        solid.quad(at(t0, edge, low0), at(t1, edge, low1), at(t1, edge, top1), at(t0, edge, top0), lit);
        solid.quad(at(t1, -edge, low1), at(t0, -edge, low0), at(t0, -edge, top0), at(t1, -edge, top1), shaded);
        if !far {
            solid.quad(at(t0, inner, top0), at(t0, edge, top0), at(t1, edge, top1), at(t1, inner, top1), shade(stone, 1.06));
            solid.quad(at(t0, -edge, top0), at(t0, -inner, top0), at(t1, -inner, top1), at(t1, -edge, top1), shade(stone, 1.06));
            upright(solid, at(t1, inner, d1), at(t0, inner, d0), at(t0, inner, top0), at(t1, inner, top1), shaded);
            upright(solid, at(t0, -inner, d0), at(t1, -inner, d1), at(t1, -inner, top1), at(t0, -inner, top0), lit);
        }
        if near && (0.0..=1.0).contains(&t0) && (0.0..=1.0).contains(&t1) {
            // the coping that oversails each parapet, and the string course at the roadway's height
            let (lip, out) = (0.0009, edge + 0.0007);
            let cope = shade(stone, 1.12);
            solid.quad(at(t0, out, top0), at(t1, out, top1), at(t1, out, top1 + lip), at(t0, out, top0 + lip), shade(cope, 0.9));
            solid.quad(at(t1, -out, top1), at(t0, -out, top0), at(t0, -out, top0 + lip), at(t1, -out, top1 + lip), shade(cope, 0.86));
            solid.quad(at(t0, out, d0 - 0.0008), at(t1, out, d1 - 0.0008), at(t1, out, d1 + 0.0006), at(t0, out, d0 + 0.0006), shade(cope, 0.94));
            solid.quad(at(t1, -out, d1 - 0.0008), at(t0, -out, d0 - 0.0008), at(t0, -out, d0 + 0.0006), at(t1, -out, d1 + 0.0006), shade(cope, 0.9));
            solid.quad(at(t0, inner - 0.0006, top0 + lip), at(t0, out, top0 + lip), at(t1, out, top1 + lip), at(t1, inner - 0.0006, top1 + lip), cope);
            solid.quad(at(t0, -out, top0 + lip), at(t0, -inner + 0.0006, top0 + lip), at(t1, -inner + 0.0006, top1 + lip), at(t1, -out, top1 + lip), cope);
        }
        if arch {
            // the underside of the arch, in shadow
            solid.quad(at(t0, edge, low0), at(t0, -edge, low0), at(t1, -edge, low1), at(t1, edge, low1), under);
        }
    }
    // the two ends, under the feet of the ramps
    let (t0, t1) = (-ramp, 1.0 + ramp);
    let (d0, d1) = (deck(t0), deck(t1));
    solid.quad(at(t0, -edge, foot), at(t0, edge, foot), at(t0, edge, d0), at(t0, -edge, d0), shade(stone, 0.9));
    solid.quad(at(t1, edge, foot), at(t1, -edge, foot), at(t1, -edge, d1), at(t1, edge, d1), shade(stone, 0.9));
    // a cutwater on each face of every pier between two arches: a wedge of stone with a sloping top
    let nose = if b.main { 0.008 } else { 0.006 };
    for a in 1..if far { 1 } else { arches } {
        let t = a as f32 * bay;
        let w = 0.085 * bay;
        let top = spring + 0.5 * rise(a).min(rise(a - 1));
        for s in [-1.0f32, 1.0] {
            let (l, r, tip) = (if s > 0.0 { t - w } else { t + w }, if s > 0.0 { t + w } else { t - w }, s * (edge + nose));
            // seen from outside its face: the left flank, the right flank and the cap
            solid.quad(at(l, s * edge, foot), at(t, tip, foot), at(t, tip, top), at(l, s * edge, top + 0.006), stone);
            solid.quad(at(t, tip, foot), at(r, s * edge, foot), at(r, s * edge, top + 0.006), at(t, tip, top), shade(stone, 0.92));
            solid.tri(at(l, s * edge, top + 0.006), at(t, tip, top), at(r, s * edge, top + 0.006), shade(stone, 1.05));
        }
    }
}

/// An upright face low0-low1-high1-high0 that may have no height at one
/// end, as the inner side of a parapet has at the foot of a ramp. A quad
/// takes its facing from its first corner, so the corners are given from the
/// end that has height: begun at the other, the face would have no facing
/// and be drawn unlit.
fn upright(solid: &mut MeshBuf, low0: Vec3, low1: Vec3, high1: Vec3, high0: Vec3, col: Rgba) {
    if high0.y - low0.y > 1e-6 {
        solid.quad(low0, low1, high1, high0, col);
    } else {
        solid.quad(low1, high1, high0, low0, col);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::Detail;

    fn bridge(half: f32, main: bool) -> RoadBridge {
        RoadBridge { at: Vec2::ZERO, along: Vec2::X, half, level: 0.0, main, detail: Detail::Play }
    }

    fn build(half: f32, main: bool) -> MeshBuf {
        let mut m = MeshBuf::default();
        road_bridge(&mut m, &bridge(half, main), &|_, _| 0.01, &Palette::standard());
        assert!(m.is_sound());
        m
    }

    /// Whether any triangle of the bridge's +z face covers the point (x, y) of that face.
    fn face_covers(m: &MeshBuf, z: f32, x: f32, y: f32) -> bool {
        m.idx.chunks(3).any(|t| {
            let p: Vec<[f32; 3]> = t.iter().map(|&i| m.pos[i as usize]).collect();
            if p.iter().any(|q| (q[2] - z).abs() > 1e-5) {
                return false;
            }
            let edge = |a: [f32; 3], b: [f32; 3]| (b[0] - a[0]) * (y - a[1]) - (b[1] - a[1]) * (x - a[0]);
            let (e0, e1, e2) = (edge(p[0], p[1]), edge(p[1], p[2]), edge(p[2], p[0]));
            (e0 >= 0.0 && e1 >= 0.0 && e2 >= 0.0) || (e0 <= 0.0 && e1 <= 0.0 && e2 <= 0.0)
        })
    }

    #[test]
    fn a_wider_river_gets_more_arches() {
        let pal = Palette::standard();
        let under = |m: &MeshBuf| m.col.iter().filter(|c| **c == *pal.bridges.arch_shadow).count() / 4;
        for (half, arches) in [(0.012, 1), (0.02, 1), (0.06, 2), (0.1, 3)] {
            assert_eq!(bridge(half, false).arches(), arches);
            // the underside of each arch is drawn in its flat lengths
            assert_eq!(under(&build(half, false)), arches * ARC, "a river {} wide", half * 2.0);
        }
    }

    #[test]
    fn the_arches_are_open_and_the_piers_are_stone() {
        for main in [false, true] {
            for half in [0.012, 0.025, 0.04, 0.06, 0.1, 0.15] {
                let (b, m) = (bridge(half, main), build(half, main));
                let z = b.width() * 0.5;
                let x_of = |t: f32| b.span() * (t - 0.5);
                let arches = b.arches();
                for a in 0..arches {
                    // the middle of each arch, a little above the water, is open; the crown over it is stone
                    let mid = x_of((a as f32 + 0.5) / arches as f32);
                    assert!(!face_covers(&m, z, mid, 0.004), "the arch of a bridge over {} is filled in", half * 2.0);
                    let crown = m.pos.iter().filter(|p| (p[0] - mid).abs() < 1e-4).map(|p| p[1]).fold(f32::MIN, f32::max);
                    assert!(face_covers(&m, z, mid, crown - 0.004));
                }
                // the abutments and the piers stand in the water
                for a in 0..=arches {
                    let x = x_of((a as f32 / arches as f32).clamp(0.01, 0.99));
                    assert!(face_covers(&m, z, x, 0.004), "no stone at {x} of a bridge over {}", half * 2.0);
                }
            }
        }
    }

    #[test]
    fn it_spans_the_river_and_humps_over_it() {
        let m = build(0.04, true);
        let (lo, hi) = m.bounds().unwrap();
        // bank to bank, and a ramp at each end
        assert!((hi.x - lo.x - 0.115 - 2.0 * APPROACH).abs() < 1e-5);
        // the crown is above the ends, which meet the road on the banks
        assert!(hi.y > 0.022 + 0.012);
        // a main road's bridge is wider than a lane's
        let lane = build(0.04, false).bounds().unwrap();
        assert!(hi.z - lo.z > lane.1.z - lane.0.z);
        // the roadway is the road's own colour, between two parapets that stand above it
        let pal = Palette::standard();
        let top_of = |col: crate::colour::Rgba| m.pos.iter().zip(&m.col).filter(|(_, c)| **c == col).map(|(p, _)| p[1]).fold(f32::MIN, f32::max);
        let roadway = top_of(*pal.ground.road_gravel);
        assert!(roadway > 0.03 && (hi.y - roadway - PARAPET.0).abs() < 1e-5);
    }

    #[test]
    fn its_faces_look_outward() {
        for half in [0.02, 0.06, 0.1, 0.15] {
            let b = bridge(half, true);
            let m = build(half, true);
            let pal = Palette::standard();
            // every face has a facing, the parapets' inner sides on the ramps too, where they run down to nothing
            assert!(m.nrm.iter().all(|n| (Vec3::from_array(*n).length() - 1.0).abs() < 1e-4), "a face of a bridge over {} has no facing", half * 2.0);
            for t in m.idx.chunks(3) {
                let (p, n) = (Vec3::from_array(m.pos[t[0] as usize]), Vec3::from_array(m.nrm[t[0] as usize]));
                let col = m.col[t[0] as usize];
                if col == *pal.ground.road_gravel {
                    // up, and never steeper than a cart can climb
                    assert!(n.y > 0.9, "the roadway looks {n}");
                } else if col == *pal.bridges.arch_shadow {
                    assert!(n.y < 0.0, "the underside of an arch looks {n}");
                } else if p.z.abs() > b.width() * 0.5 - 1e-5 && n.z.abs() > 0.5 {
                    assert!(n.z * p.z > 0.0, "a face looks inward at {p}");
                }
            }
        }
    }

    #[test]
    fn the_roadway_a_cart_reads_is_the_roadway_drawn() {
        // over a flat bank and a sloping one, at each level of detail, every corner of the
        // drawn roadway lies on the roadway the bridge gives out, between its parapets
        let pal = Palette::standard();
        let slope = |x: f32, _: f32| 0.1 + 0.08 * (x - 3.0).max(0.0);
        for detail in [Detail::Far, Detail::Play, Detail::Near] {
            for half in [0.012, 0.04, 0.1] {
                let b = RoadBridge { at: Vec2::new(3.0, 4.0), along: Vec2::new(0.6, 0.8), half, level: 0.09, main: true, detail };
                let mut m = MeshBuf::default();
                road_bridge(&mut m, &b, &slope, &pal);
                let way = b.roadway(&slope);
                assert!(way.windows(2).all(|w| w[0].0 < w[1].0));
                assert!((way[0].0 + b.ramp()).abs() < 1e-6 && (way[way.len() - 1].0 - 1.0 - b.ramp()).abs() < 1e-6);
                let height = |t: f32| {
                    let i = way.partition_point(|p| p.0 < t).clamp(1, way.len() - 1);
                    let ((t0, h0), (t1, h1)) = (way[i - 1], way[i]);
                    h0 + (h1 - h0) * ((t - t0) / (t1 - t0)).clamp(0.0, 1.0)
                };
                let mut corners = 0;
                for (p, c) in m.pos.iter().zip(&m.col) {
                    if *c != *pal.ground.road_gravel {
                        continue;
                    }
                    let p = Vec3::from_array(*p);
                    let t = b.t_of(Vec2::new(p.x, p.z));
                    let out = (Vec2::new(p.x, p.z) - b.point(t)).length();
                    assert!((p.y - height(t)).abs() < 1e-5, "{detail:?} over {half}: a corner of the roadway at {t} is at {} and the roadway gives {}", p.y, height(t));
                    assert!((out - b.inner_half()).abs() < 1e-5);
                    corners += 1;
                }
                assert!(corners >= 4 * (way.len() - 1));
            }
        }
    }

    #[test]
    fn it_follows_its_road_and_its_banks() {
        // the same bridge on a road running another way is the same bridge turned
        let pal = Palette::standard();
        let mut a = MeshBuf::default();
        let mut c = MeshBuf::default();
        road_bridge(&mut a, &RoadBridge { at: Vec2::new(3.0, 4.0), along: Vec2::X, half: 0.03, level: 0.1, main: false, detail: Detail::Play }, &|_, _| 0.12, &pal);
        road_bridge(&mut c, &RoadBridge { at: Vec2::new(3.0, 4.0), along: Vec2::Y, half: 0.03, level: 0.1, main: false, detail: Detail::Play }, &|_, _| 0.12, &pal);
        let (la, ha) = a.bounds().unwrap();
        let (lc, hc) = c.bounds().unwrap();
        assert!(((ha.x - la.x) - (hc.z - lc.z)).abs() < 1e-5 && ((ha.z - la.z) - (hc.x - lc.x)).abs() < 1e-5);
        assert!((ha.y - hc.y).abs() < 1e-6 && la.y < 0.1 && ha.y > 0.13);
        // a higher bank lifts that end of the roadway, and the bridge climbs to it
        let mut high = MeshBuf::default();
        road_bridge(&mut high, &RoadBridge { at: Vec2::new(3.0, 4.0), along: Vec2::X, half: 0.03, level: 0.1, main: false, detail: Detail::Play }, &|x, _| if x > 3.0 { 0.2 } else { 0.12 }, &pal);
        assert!(high.is_sound() && high.bounds().unwrap().1.y > ha.y + 0.05);
        let top_at = |x: f32| high.pos.iter().filter(|p| (p[0] - x).abs() < 1e-4).map(|p| p[1]).fold(f32::MIN, f32::max);
        assert!(top_at(3.0 - 0.0475) < 0.15 && top_at(3.0 + 0.0475) > 0.21);
    }
}
