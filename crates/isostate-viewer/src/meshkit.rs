//! A tiny kit for building low-poly, flat-shaded, vertex-coloured meshes.
//! Everything of one kind (all trees, all houses) is merged into one mesh,
//! rebuilt when the year changes.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::math::{Quat, Vec3};

#[derive(Default)]
pub struct Builder {
    pub pos: Vec<[f32; 3]>,
    pub nrm: Vec<[f32; 3]>,
    pub col: Vec<[f32; 4]>,
    pub idx: Vec<u32>,
}

/// sRGB hex colour to linear RGBA, which is what vertex colours expect.
pub fn hex(c: u32) -> [f32; 4] {
    let f = |v: u32| {
        let s = v as f32 / 255.0;
        if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    [f((c >> 16) & 255), f((c >> 8) & 255), f(c & 255), 1.0]
}

pub fn shade(c: [f32; 4], k: f32) -> [f32; 4] {
    [c[0] * k, c[1] * k, c[2] * k, c[3]]
}

pub fn lerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t, 1.0]
}

impl Builder {
    pub fn is_empty(&self) -> bool {
        self.idx.is_empty()
    }

    /// A flat triangle (counter-clockwise when seen from outside).
    pub fn tri(&mut self, a: Vec3, b: Vec3, c: Vec3, col: [f32; 4]) {
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
    pub fn quad(&mut self, a: Vec3, b: Vec3, c: Vec3, d: Vec3, col: [f32; 4]) {
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
    pub fn block(&mut self, base: Vec3, w: f32, d: f32, h: f32, yaw: f32, wall: [f32; 4], top: Option<[f32; 4]>, sink: f32) {
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
    pub fn gable(&mut self, base: Vec3, w: f32, d: f32, h0: f32, rise: f32, yaw: f32, over: f32, roof: [f32; 4], gable_col: [f32; 4]) {
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
    pub fn frustum(&mut self, base: Vec3, r0: f32, r1: f32, h: f32, seg: usize, col: [f32; 4], cap: Option<[f32; 4]>) {
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
    pub fn ball(&mut self, centre: Vec3, r: f32, hy: f32, seg: usize, rings: usize, col: [f32; 4], jitter_seed: f32) {
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
    pub fn beam(&mut self, a: Vec3, b: Vec3, t: f32, col: [f32; 4]) {
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
    pub fn ribbon(&mut self, pts: &[Vec3], width: f32, col: [f32; 4]) {
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
    pub fn ribbon_var(&mut self, pts: &[Vec3], widths: &[f32], col: [f32; 4]) {
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

    /// Append another builder's triangles.
    pub fn append(&mut self, other: Builder) {
        let base = self.pos.len() as u32;
        self.pos.extend(other.pos);
        self.nrm.extend(other.nrm);
        self.col.extend(other.col);
        self.idx.extend(other.idx.into_iter().map(|i| i + base));
    }

    pub fn into_mesh(self) -> Mesh {
        let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        m.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.pos);
        m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.nrm);
        m.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.col);
        m.insert_indices(Indices::U32(self.idx));
        m
    }
}
