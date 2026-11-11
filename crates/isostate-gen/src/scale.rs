//! The scale: the one set of sizes every generator answers to, and the rule
//! that measures a generator against it.
//!
//! Each kind of thing on the island was once built with a kit of its own and
//! its own idea of how fine detail goes, and the picture swung from one to
//! the other. The houses are the reference. What they are sized in is
//! written here as constants (`STOREY`, `BAY`, the grain of each level), and
//! `measure` reads the same quantities back off any mesh, so that a test can
//! hold every generator in `registry` to the bands the houses set (`LEVELS`).
//! README.md, "The scale", says it in words.

use glam::Vec3;

use crate::kit::Detail;
use crate::mesh::MeshBuf;
use crate::palette::Palette;

/// The height of one storey, of every building: floor to floor, and from the
/// ground to the eaves of a building of one storey. (Until November 2026 a
/// works' storey was 0.022 and a church's, a station's and a barn's 0.024.)
/// A hall (a forge, an engine house, a barn, a nave: one tall space) is as
/// high as it needs to be, and its openings are still sized by this.
pub const STOREY: f32 = 0.0205;

/// The module openings repeat at along a wall: the width of one bay. A
/// cottage is two or three bays wide, a mill as many as its length holds.
/// Bays run from one module to one and a half.
pub const BAY: f32 = 0.014;

/// The longest run of wall, in modules, that may be left with nothing on it
/// at the play and the near levels: no opening, no pier, no lean-to, nothing
/// standing against it.
pub const BLANK: f32 = 4.0;

/// What one level of detail may and must draw, as measured off the houses.
#[derive(Clone, Copy, Debug)]
pub struct Level {
    /// the smallest feature there may be: nothing thinner is drawn at this level
    pub finest: f32,
    /// the smallest feature there must be: a thing whose finest part is
    /// coarser than this has less to it than a house has at this level
    pub coarsest: f32,
    /// the triangles to one bay of one storey of wall: at least, at most
    pub wall: (f32, f32),
    /// the triangles to one bay by one bay of roof: at least, at most
    pub roof: (f32, f32),
}

/// The three levels (`Detail::Far`, `Play`, `Near`), in that order.
///
/// Far: walls and roofs, nothing let into them or standing on them. Play:
/// openings as flat panels, stacks, wings and lean-tos, what stands in the
/// yard as plain blocks. Near: openings let into the wall with reveals,
/// sills and glazing bars, eaves with an edge, stacks with their heads,
/// copings, piers, and what stands in the yard in full.
pub const LEVELS: [Level; 3] = [
    Level { finest: 0.0012, coarsest: f32::INFINITY, wall: (0.05, 1.5), roof: (0.03, 1.5) },
    Level { finest: 0.0012, coarsest: 0.006, wall: (0.6, 4.5), roof: (0.25, 2.5) },
    Level { finest: 0.00035, coarsest: 0.001, wall: (4.0, 26.0), roof: (0.7, 10.0) },
];

impl Level {
    pub fn of(detail: Detail) -> &'static Level {
        &LEVELS[detail as usize]
    }
}

/// What `measure` reads off a mesh.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Measure {
    pub triangles: usize,
    /// the smallest feature: the least width of any triangle (its shortest altitude)
    pub smallest: f32,
    /// triangles to a bay of a storey of wall (faces that stand upright and reach above a child's height)
    pub wall: f32,
    /// triangles to a bay by a bay of roof (faces that look up, above the ground)
    pub roof: f32,
    /// the area of wall and of roof those were counted over, in bays of a storey and bays by bays
    pub wall_area: f32,
    pub roof_area: f32,
    /// the longest run of wall with nothing on it or before it, in modules
    pub blank: f32,
    /// how far apart windows of one size stand one above another, where any do
    pub storey: Option<f32>,
}

/// A face stands upright if its facing leans no more than this from level.
const UPRIGHT: f32 = 0.3;
/// Wall below this height is a yard wall, a step or a bed, not the wall of a building.
const LOW: f32 = 0.4 * STOREY;

struct Tri {
    p: [Vec3; 3],
    n: Vec3,
    area: f32,
}

fn tris(b: &MeshBuf) -> Vec<Tri> {
    b.idx
        .chunks(3)
        .filter_map(|t| {
            let p = [0, 1, 2].map(|i| Vec3::from_array(b.pos[t[i] as usize]));
            let c = (p[1] - p[0]).cross(p[2] - p[0]);
            let area = c.length() * 0.5;
            (area > 1e-10).then(|| Tri { p, n: c / (area * 2.0), area })
        })
        .collect()
}

/// Measure a mesh built on level ground at the origin against the scale.
pub fn measure(b: &MeshBuf, pal: &Palette) -> Measure {
    let ts = tris(b);
    let ground = b.pos.iter().map(|p| p[1]).fold(f32::MAX, f32::min).max(-0.004);
    let mut m = Measure { triangles: ts.len(), smallest: f32::MAX, ..Default::default() };
    let (mut wall_n, mut wall_a, mut roof_n, mut roof_a) = (0usize, 0.0f32, 0usize, 0.0f32);
    for t in &ts {
        let longest = (0..3).map(|i| (t.p[(i + 1) % 3] - t.p[i]).length()).fold(0.0f32, f32::max);
        m.smallest = m.smallest.min(2.0 * t.area / longest);
        let top = t.p.iter().map(|p| p.y).fold(f32::MIN, f32::max) - ground;
        if t.n.y.abs() < UPRIGHT && top > LOW {
            wall_n += 1;
            wall_a += t.area;
        } else if t.n.y >= UPRIGHT && top > LOW {
            roof_n += 1;
            roof_a += t.area;
        }
    }
    m.wall_area = wall_a / (BAY * STOREY);
    m.roof_area = roof_a / (BAY * BAY);
    m.wall = if m.wall_area > 0.0 { wall_n as f32 / m.wall_area } else { 0.0 };
    m.roof = if m.roof_area > 0.0 { roof_n as f32 / m.roof_area } else { 0.0 };
    m.blank = blank(&ts, ground) / BAY;
    m.storey = stacked(b, pal);
    m
}

/// The longest run of bare wall: over every big upright face, the longest
/// stretch along it that no other face stands on or before.
fn blank(ts: &[Tri], ground: f32) -> f32 {
    /// what stands this near before a wall is on it or against it
    const BEFORE: f32 = 0.012;
    let mut worst = 0.0f32;
    for (i, t) in ts.iter().enumerate() {
        if t.n.y.abs() >= UPRIGHT {
            continue;
        }
        let along = Vec3::new(-t.n.z, 0.0, t.n.x).normalize_or_zero();
        let u = t.p.map(|p| p.dot(along));
        let (u0, u1) = (u.iter().copied().fold(f32::MAX, f32::min), u.iter().copied().fold(f32::MIN, f32::max));
        let (y0, y1) = (t.p.iter().map(|p| p.y).fold(f32::MAX, f32::min), t.p.iter().map(|p| p.y).fold(f32::MIN, f32::max));
        // a wall of a building: long, and standing well above the ground for much of a storey
        if u1 - u0 <= worst.max(2.0 * BAY) || y1 - ground < LOW * 2.0 || y1 - y0 < 0.6 * STOREY {
            continue;
        }
        // the height the wall has along its length: a triangle is half of a face, so
        // take the band of its middle half, where both halves of the face overlap
        let (ya, yb) = (y0 + (y1 - y0) * 0.25, y0 + (y1 - y0) * 0.75);
        let d = t.p[0].dot(t.n);
        let mut cover: Vec<(f32, f32)> = Vec::new();
        for (j, o) in ts.iter().enumerate() {
            if i == j {
                continue;
            }
            let off = o.p.map(|p| p.dot(t.n) - d);
            let (near, far) = (off.iter().copied().fold(f32::MAX, f32::min), off.iter().copied().fold(f32::MIN, f32::max));
            // on the wall or before it; the wall's own other half lies in its plane and is not on it
            if far < 2e-4 || near > BEFORE || (far < 3e-4 && near > -3e-4 && o.n.dot(t.n) > 0.99) {
                continue;
            }
            let (oy0, oy1) = (o.p.iter().map(|p| p.y).fold(f32::MAX, f32::min), o.p.iter().map(|p| p.y).fold(f32::MIN, f32::max));
            if oy1 < ya || oy0 > yb {
                continue;
            }
            let ou = o.p.map(|p| p.dot(along));
            cover.push((ou.iter().copied().fold(f32::MAX, f32::min), ou.iter().copied().fold(f32::MIN, f32::max)));
        }
        cover.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut at = u0;
        let mut gap = 0.0f32;
        for (a, b) in cover {
            if a > at {
                gap = gap.max(a.min(u1) - at);
            }
            at = at.max(b);
            if at >= u1 {
                break;
            }
        }
        gap = gap.max(u1 - at);
        worst = worst.max(gap);
    }
    worst
}

/// How far apart windows of one size stand one above another: the middle
/// such distance, from the flat panes of glass on upright faces (which is
/// how the play level draws a window), or nothing if no two are stacked.
fn stacked(b: &MeshBuf, pal: &Palette) -> Option<f32> {
    let glass = *pal.glass.window;
    // every upright quad of glass: its plane's facing, its middle along the wall, its foot, width and height
    let mut panes: Vec<(Vec3, f32, f32, f32, f32, f32)> = Vec::new();
    for q in b.idx.chunks(6) {
        if q.len() < 6 || q.iter().any(|&i| b.col[i as usize] != glass) {
            continue;
        }
        let mut c: Vec<u32> = q.to_vec();
        c.sort_unstable();
        c.dedup();
        if c.len() != 4 {
            continue;
        }
        let p: Vec<Vec3> = c.iter().map(|&i| Vec3::from_array(b.pos[i as usize])).collect();
        let n = Vec3::from_array(b.nrm[c[0] as usize]);
        if n.y.abs() >= UPRIGHT {
            continue;
        }
        let along = Vec3::new(-n.z, 0.0, n.x).normalize_or_zero();
        let u: Vec<f32> = p.iter().map(|v| v.dot(along)).collect();
        let (u0, u1) = (u.iter().copied().fold(f32::MAX, f32::min), u.iter().copied().fold(f32::MIN, f32::max));
        let (y0, y1) = (p.iter().map(|v| v.y).fold(f32::MAX, f32::min), p.iter().map(|v| v.y).fold(f32::MIN, f32::max));
        // (a ground floor's windows have sills of their own: the storeys above it are what is measured)
        if y0 > 0.9 * STOREY {
            panes.push((n, p[0].dot(n), (u0 + u1) * 0.5, y0, u1 - u0, y1 - y0));
        }
    }
    let mut gaps: Vec<f32> = Vec::new();
    for (i, a) in panes.iter().enumerate() {
        // the nearest pane of the same size above this one in the same wall
        let above = panes
            .iter()
            .enumerate()
            .filter(|(j, o)| *j != i && o.0.dot(a.0) > 0.999 && (o.1 - a.1).abs() < 1e-4 && (o.2 - a.2).abs() < 5e-4 && (o.4 - a.4).abs() < 2e-4 && (o.5 - a.5).abs() < 2e-4 && o.3 > a.3 + 1e-4)
            .map(|(_, o)| o.3 - a.3)
            .fold(f32::MAX, f32::min);
        if above < f32::MAX {
            gaps.push(above);
        }
    }
    gaps.sort_by(f32::total_cmp);
    gaps.get(gaps.len() / 2).copied()
}

/// What is wrong with a measured building at a level, if anything: one line for each fault.
pub fn faults(m: &Measure, detail: Detail) -> Vec<String> {
    let lv = Level::of(detail);
    let mut out = Vec::new();
    if m.smallest < lv.finest {
        out.push(format!("its smallest feature is {:.5}, finer than the {} level's {:.5}", m.smallest, detail.label(), lv.finest));
    }
    if m.smallest > lv.coarsest {
        out.push(format!("its smallest feature is {:.5}, coarser than the {} level asks ({:.5})", m.smallest, detail.label(), lv.coarsest));
    }
    if m.wall_area > 2.0 && (m.wall < lv.wall.0 || m.wall > lv.wall.1) {
        out.push(format!("{:.1} triangles to a bay of a storey of wall, outside {} to {}", m.wall, lv.wall.0, lv.wall.1));
    }
    if m.roof_area > 2.0 && (m.roof < lv.roof.0 || m.roof > lv.roof.1) {
        out.push(format!("{:.2} triangles to a bay by a bay of roof, outside {} to {}", m.roof, lv.roof.0, lv.roof.1));
    }
    if detail != Detail::Far && m.blank > BLANK {
        out.push(format!("a wall is bare for {:.1} modules (no more than {BLANK} may be)", m.blank));
    }
    if let Some(s) = m.storey {
        let n = (s / STOREY).round().max(1.0);
        if (s - n * STOREY).abs() > 0.02 * STOREY {
            out.push(format!("its windows stand {s:.4} apart one above another: a storey is {STOREY}"));
        }
    }
    out
}

/// A measure in a line or two, for a label on the lab's `scale` sheet.
pub fn words(m: &Measure) -> String {
    format!("{} tris, finest {:.1} mm\nwall {:.1}, roof {:.1}, bare {:.1}", m.triangles, m.smallest * 1000.0, m.wall, m.roof, m.blank)
}
