//! A small buffer of triangles and the shapes generators build from: low-poly,
//! flat-shaded, coloured at each corner. A generator appends to a buffer; the
//! viewer merges everything of one kind (all houses, all trees) into one and
//! turns it into a mesh in one place.

use glam::{Affine3A, Quat, Vec3};

use crate::colour::{shade, Rgba};

#[derive(Default, Clone, Debug, PartialEq)]
pub struct MeshBuf {
    pub pos: Vec<[f32; 3]>,
    pub nrm: Vec<[f32; 3]>,
    pub col: Vec<Rgba>,
    pub idx: Vec<u32>,
}

impl MeshBuf {
    pub fn is_empty(&self) -> bool {
        self.idx.is_empty()
    }

    /// How many triangles there are.
    pub fn triangles(&self) -> usize {
        self.idx.len() / 3
    }

    /// A flat triangle (counter-clockwise when seen from outside).
    pub fn tri(&mut self, a: Vec3, b: Vec3, c: Vec3, col: Rgba) {
        let n = (b - a).cross(c - a).normalize_or_zero();
        let base = self.pos.len() as u32;
        for p in [a, b, c] {
            self.pos.push(p.to_array());
            self.nrm.push(n.to_array());
            self.col.push(col);
        }
        self.idx.extend_from_slice(&[base, base + 1, base + 2]);
    }

    /// A flat quad a-b-c-d, counter-clockwise from outside.
    pub fn quad(&mut self, a: Vec3, b: Vec3, c: Vec3, d: Vec3, col: Rgba) {
        let n = (b - a).cross(d - a).normalize_or_zero();
        let base = self.pos.len() as u32;
        for p in [a, b, c, d] {
            self.pos.push(p.to_array());
            self.nrm.push(n.to_array());
            self.col.push(col);
        }
        self.idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// A box standing on `base` (centre of its bottom face), rotated by `yaw`.
    /// The bottom is left open; walls reach `sink` below the base so slopes do not show gaps.
    #[allow(clippy::too_many_arguments)]
    pub fn block(&mut self, base: Vec3, w: f32, d: f32, h: f32, yaw: f32, wall: Rgba, top: Option<Rgba>, sink: f32) {
        let q = Quat::from_rotation_y(yaw);
        let (hw, hd) = (w * 0.5, d * 0.5);
        let p = |x: f32, y: f32, z: f32| base + q * Vec3::new(x, y, z);
        let y0 = -sink;
        let c = [p(-hw, y0, -hd), p(hw, y0, -hd), p(hw, y0, hd), p(-hw, y0, hd)];
        let t = [p(-hw, h, -hd), p(hw, h, -hd), p(hw, h, hd), p(-hw, h, hd)];
        // walls: a slightly darker shade on two sides fakes soft light
        self.quad(c[3], c[2], t[2], t[3], wall); // +z
        self.quad(c[2], c[1], t[1], t[2], shade(wall, 0.93)); // +x
        self.quad(c[1], c[0], t[0], t[1], shade(wall, 0.86)); // -z
        self.quad(c[0], c[3], t[3], t[0], shade(wall, 0.95)); // -x
        if let Some(tc) = top {
            self.quad(t[3], t[2], t[1], t[0], tc);
        }
    }

    /// A pitched roof on a box of size w x d whose top is at `base.y + h0`; ridge along local x.
    #[allow(clippy::too_many_arguments)]
    pub fn gable(&mut self, base: Vec3, w: f32, d: f32, h0: f32, rise: f32, yaw: f32, over: f32, roof: Rgba, gable_col: Rgba) {
        let q = Quat::from_rotation_y(yaw);
        let p = |x: f32, y: f32, z: f32| base + q * Vec3::new(x, y, z);
        let (hw, hd) = (w * 0.5 + over, d * 0.5 + over);
        let (ew, ed) = (w * 0.5, d * 0.5);
        let r0 = p(-hw, h0 + rise, 0.0);
        let r1 = p(hw, h0 + rise, 0.0);
        let e = h0 - over * 0.6;
        // two roof slopes
        self.quad(p(-hw, e, hd), p(hw, e, hd), r1, r0, roof);
        self.quad(p(hw, e, -hd), p(-hw, e, -hd), r0, r1, shade(roof, 0.82));
        // gable ends
        self.tri(p(ew, h0, ed), p(ew, h0, -ed), p(ew, h0 + rise - over * 0.2, 0.0), gable_col);
        self.tri(p(-ew, h0, -ed), p(-ew, h0, ed), p(-ew, h0 + rise - over * 0.2, 0.0), shade(gable_col, 0.9));
    }

    /// A tapered column: radius r0 at the bottom, r1 at the top, `seg` sides.
    /// With r1 at nothing it is a cone or, with four sides, a spire or a hipped roof.
    #[allow(clippy::too_many_arguments)]
    pub fn frustum(&mut self, base: Vec3, r0: f32, r1: f32, h: f32, seg: usize, col: Rgba, cap: Option<Rgba>) {
        let ring = |r: f32, y: f32, i: usize| {
            let a = i as f32 / seg as f32 * std::f32::consts::TAU;
            base + Vec3::new(a.cos() * r, y, a.sin() * r)
        };
        for i in 0..seg {
            let (a0, a1) = (ring(r0, -0.02, i), ring(r0, -0.02, i + 1));
            let (b0, b1) = (ring(r1, h, i), ring(r1, h, i + 1));
            let k = 0.88 + 0.12 * ((i as f32 / seg as f32 * std::f32::consts::TAU + 0.8).cos() * 0.5 + 0.5);
            if r1 > 1e-4 {
                self.quad(a1, a0, b0, b1, shade(col, k));
            } else {
                self.tri(a1, a0, base + Vec3::new(0.0, h, 0.0), shade(col, k));
            }
        }
        if let Some(cc) = cap {
            if r1 > 1e-4 {
                let top = base + Vec3::new(0.0, h, 0.0);
                for i in 0..seg {
                    self.tri(ring(r1, h, i), top, ring(r1, h, i + 1), cc);
                }
            }
        }
    }

    /// A low-poly ball: `seg` around, `rings` up, squashed to height `hy`.
    #[allow(clippy::too_many_arguments)]
    pub fn ball(&mut self, centre: Vec3, r: f32, hy: f32, seg: usize, rings: usize, col: Rgba, jitter_seed: f32) {
        let pt = |i: usize, j: usize| {
            let th = j as f32 / rings as f32 * std::f32::consts::PI;
            let ph = i as f32 / seg as f32 * std::f32::consts::TAU + jitter_seed;
            let wob = 1.0 + 0.08 * ((ph * 3.0 + th * 2.0 + jitter_seed * 7.0).sin());
            centre + Vec3::new(th.sin() * ph.cos() * r * wob, -th.cos() * hy, th.sin() * ph.sin() * r * wob)
        };
        for j in 0..rings {
            for i in 0..seg {
                let (a, b, c, d) = (pt(i, j), pt(i + 1, j), pt(i + 1, j + 1), pt(i, j + 1));
                // lighter on top, darker underneath: cheap ambient occlusion
                let k = 0.72 + 0.4 * (j as f32 + 0.5) / rings as f32;
                if j == 0 {
                    self.tri(a, d, c, shade(col, k));
                } else if j == rings - 1 {
                    self.tri(b, a, d, shade(col, k));
                } else {
                    self.quad(b, a, d, c, shade(col, k));
                }
            }
        }
    }

    /// A square-section beam from `a` to `b` (headframe legs, cranes).
    pub fn beam(&mut self, a: Vec3, b: Vec3, t: f32, col: Rgba) {
        let dir = (b - a).normalize_or_zero();
        let up = if dir.y.abs() > 0.9 { Vec3::X } else { Vec3::Y };
        let u = dir.cross(up).normalize_or_zero() * (t * 0.5);
        let v = dir.cross(u).normalize_or_zero() * (t * 0.5);
        let ca = [a + u + v, a - u + v, a - u - v, a + u - v];
        let cb = [b + u + v, b - u + v, b - u - v, b + u - v];
        for i in 0..4 {
            let j = (i + 1) % 4;
            let k = 0.85 + 0.15 * (i as f32 / 3.0);
            self.quad(ca[i], ca[j], cb[j], cb[i], shade(col, k));
        }
    }

    /// A flat ribbon along a polyline; points carry their own heights.
    pub fn ribbon(&mut self, pts: &[Vec3], width: f32, col: Rgba) {
        if pts.len() < 2 {
            return;
        }
        let base = self.pos.len() as u32;
        for (i, p) in pts.iter().enumerate() {
            let prev = if i > 0 { pts[i - 1] } else { *p };
            let next = if i + 1 < pts.len() { pts[i + 1] } else { *p };
            let dir = Vec3::new(next.x - prev.x, 0.0, next.z - prev.z).normalize_or_zero();
            let side = Vec3::new(-dir.z, 0.0, dir.x) * (width * 0.5);
            self.pos.push((*p + side).to_array());
            self.pos.push((*p - side).to_array());
            for _ in 0..2 {
                self.nrm.push([0.0, 1.0, 0.0]);
                self.col.push(col);
            }
        }
        for i in 0..(pts.len() as u32 - 1) {
            let a = base + i * 2;
            self.idx.extend_from_slice(&[a, a + 2, a + 1, a + 1, a + 2, a + 3]);
        }
    }

    /// A flat ribbon whose width changes along it.
    pub fn ribbon_var(&mut self, pts: &[Vec3], widths: &[f32], col: Rgba) {
        if pts.len() < 2 {
            return;
        }
        let base = self.pos.len() as u32;
        for (i, p) in pts.iter().enumerate() {
            let prev = if i > 0 { pts[i - 1] } else { *p };
            let next = if i + 1 < pts.len() { pts[i + 1] } else { *p };
            let dir = Vec3::new(next.x - prev.x, 0.0, next.z - prev.z).normalize_or_zero();
            let side = Vec3::new(-dir.z, 0.0, dir.x) * (widths[i.min(widths.len() - 1)] * 0.5);
            self.pos.push((*p + side).to_array());
            self.pos.push((*p - side).to_array());
            for _ in 0..2 {
                self.nrm.push([0.0, 1.0, 0.0]);
                self.col.push(col);
            }
        }
        for i in 0..(pts.len() as u32 - 1) {
            let a = base + i * 2;
            self.idx.extend_from_slice(&[a, a + 2, a + 1, a + 1, a + 2, a + 3]);
        }
    }

    /// Append another buffer's triangles.
    pub fn append(&mut self, other: MeshBuf) {
        let base = self.pos.len() as u32;
        self.pos.extend(other.pos);
        self.nrm.extend(other.nrm);
        self.col.extend(other.col);
        self.idx.extend(other.idx.into_iter().map(|i| i + base));
    }

    /// Append a copy of another buffer's triangles, leaving it as it is.
    pub fn merge(&mut self, other: &MeshBuf) {
        let base = self.pos.len() as u32;
        self.pos.extend_from_slice(&other.pos);
        self.nrm.extend_from_slice(&other.nrm);
        self.col.extend_from_slice(&other.col);
        self.idx.extend(other.idx.iter().map(|i| i + base));
    }

    /// Move, turn or scale everything in the buffer. Normals turn with it.
    /// (The game does not use this: generators write where they stand, so
    /// that every corner is worked out the same way each time. It is for the
    /// lab and for building a thing once and standing copies about.)
    pub fn transform(&mut self, t: Affine3A) {
        // normals go by the inverse transpose, so an uneven scale keeps them square to their faces
        let n = t.matrix3.inverse().transpose();
        for p in &mut self.pos {
            *p = t.transform_point3(Vec3::from_array(*p)).to_array();
        }
        for v in &mut self.nrm {
            *v = (n * glam::Vec3A::from_array(*v)).normalize_or_zero().to_array();
        }
    }

    /// The least and greatest corner of the box round everything, if there is anything.
    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        let mut it = self.pos.iter().map(|p| Vec3::from_array(*p));
        let first = it.next()?;
        Some(it.fold((first, first), |(lo, hi), p| (lo.min(p), hi.max(p))))
    }

    /// Every number in the buffer is a real one and every index points at a corner.
    pub fn is_sound(&self) -> bool {
        let n = self.pos.len();
        self.nrm.len() == n
            && self.col.len() == n
            && self.idx.len() % 3 == 0
            && self.idx.iter().all(|&i| (i as usize) < n)
            && self.pos.iter().chain(&self.nrm).all(|p| p.iter().all(|v| v.is_finite()))
            && self.col.iter().all(|c| c.iter().all(|v| v.is_finite()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Rgba = [1.0, 0.0, 0.0, 1.0];

    #[test]
    fn a_block_has_four_walls_and_perhaps_a_top() {
        let mut b = MeshBuf::default();
        b.block(Vec3::ZERO, 2.0, 1.0, 3.0, 0.0, RED, None, 0.5);
        assert_eq!(b.triangles(), 8);
        b.block(Vec3::ZERO, 2.0, 1.0, 3.0, 0.0, RED, Some(RED), 0.5);
        assert_eq!(b.triangles(), 8 + 10);
        assert!(b.is_sound());
        let (lo, hi) = b.bounds().unwrap();
        assert_eq!((lo, hi), (Vec3::new(-1.0, -0.5, -0.5), Vec3::new(1.0, 3.0, 0.5)));
    }

    #[test]
    fn faces_look_outward() {
        // every face of a closed-topped box and of a ball turns away from the middle
        let mut b = MeshBuf::default();
        b.block(Vec3::ZERO, 1.0, 1.0, 1.0, 0.3, RED, Some(RED), 0.0);
        b.ball(Vec3::new(0.0, 0.5, 0.0), 0.4, 0.4, 6, 4, RED, 0.2);
        b.frustum(Vec3::ZERO, 0.3, 0.2, 1.0, 6, RED, Some(RED));
        let mid = Vec3::new(0.0, 0.5, 0.0);
        for t in b.idx.chunks(3) {
            let p = Vec3::from_array(b.pos[t[0] as usize]);
            let n = Vec3::from_array(b.nrm[t[0] as usize]);
            assert!(n.dot(p - mid) > -1e-4, "a face looks inward at {p}");
        }
    }

    #[test]
    fn append_and_merge_keep_the_indices_right() {
        let mut a = MeshBuf::default();
        a.tri(Vec3::ZERO, Vec3::X, Vec3::Z, RED);
        let mut b = MeshBuf::default();
        b.quad(Vec3::ZERO, Vec3::X, Vec3::ONE, Vec3::Z, RED);
        let mut both = a.clone();
        both.merge(&b);
        a.append(b);
        assert_eq!(a, both);
        assert_eq!(a.idx, vec![0, 1, 2, 3, 4, 5, 3, 5, 6]);
        assert!(a.is_sound());
    }

    #[test]
    fn a_transform_moves_points_and_turns_normals() {
        let mut b = MeshBuf::default();
        b.quad(Vec3::ZERO, Vec3::Z, Vec3::new(1.0, 0.0, 1.0), Vec3::X, RED);
        assert_eq!(b.nrm[0], [0.0, 1.0, 0.0]);
        b.transform(Affine3A::from_rotation_translation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2), Vec3::new(0.0, 5.0, 0.0)));
        let n = Vec3::from_array(b.nrm[0]);
        assert!((n - Vec3::Z).length() < 1e-5, "{n}");
        assert!((Vec3::from_array(b.pos[1]) - Vec3::new(0.0, 4.0, 0.0)).length() < 1e-5);
        // a plain move leaves the normals alone
        let mut c = b.clone();
        c.transform(Affine3A::from_translation(Vec3::X));
        assert_eq!(c.bounds().unwrap().0.x, b.bounds().unwrap().0.x + 1.0);
        assert_eq!(c.nrm, b.nrm);
    }

    #[test]
    fn an_empty_buffer_has_no_bounds() {
        assert!(MeshBuf::default().bounds().is_none());
        assert!(MeshBuf::default().is_empty());
    }
}
