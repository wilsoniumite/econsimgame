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
    Level { finest: 0.0012, coarsest: f32::INFINITY, wall: (0.02, 1.5), roof: (0.03, 2.8) },
    Level { finest: 0.0012, coarsest: 0.006, wall: (0.6, 4.0), roof: (0.04, 3.2) },
    Level { finest: 0.00035, coarsest: 0.001, wall: (4.0, 24.0), roof: (0.1, 8.0) },
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
    /// where the smallest feature and the barest wall are, for finding them
    pub smallest_at: [f32; 3],
    pub blank_at: [f32; 3],
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

/// What the kit has built: the walls and roofs every building is put
/// together from tell this tally of their area and their triangles as they
/// are built, so that a building's walls and roofs can be measured apart
/// from what stands about it (its chimneys, its heaps, its yard). It is
/// kept for each thread, changes nothing that is drawn, and is read by
/// `take_tally`: take it before building a thing and again after.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tally {
    pub wall_tris: usize,
    /// in map units squared
    pub wall_area: f32,
    pub roof_tris: usize,
    pub roof_area: f32,
}

thread_local! {
    static TALLY: std::cell::Cell<Tally> = const { std::cell::Cell::new(Tally { wall_tris: 0, wall_area: 0.0, roof_tris: 0, roof_area: 0.0 }) };
}

/// Walls of an area were built in so many triangles.
pub(crate) fn tally_wall(area: f32, tris: usize) {
    TALLY.with(|t| {
        let mut v = t.get();
        v.wall_tris += tris;
        v.wall_area += area;
        t.set(v);
    });
}

/// Roof of an area was built in so many triangles.
pub(crate) fn tally_roof(area: f32, tris: usize) {
    TALLY.with(|t| {
        let mut v = t.get();
        v.roof_tris += tris;
        v.roof_area += area;
        t.set(v);
    });
}

/// The tally as it stands.
pub(crate) fn peek_tally() -> Tally {
    TALLY.with(|t| t.get())
}

/// What the kit has built on this thread since the tally was last taken.
pub fn take_tally() -> Tally {
    TALLY.with(|t| t.replace(Tally::default()))
}

/// Measure a thing built on level ground at the origin against the scale:
/// its mesh, and what the kit's tally says of its walls and roofs.
pub fn measure(b: &MeshBuf, tally: &Tally, pal: &Palette) -> Measure {
    let ts = tris(b);
    // (the thing stands on level ground at the origin)
    let ground = 0.0;
    let mut m = Measure { triangles: ts.len(), smallest: f32::MAX, ..Default::default() };
    for t in &ts {
        let longest = (0..3).map(|i| (t.p[(i + 1) % 3] - t.p[i]).length()).fold(0.0f32, f32::max);
        if 2.0 * t.area / longest < m.smallest {
            (m.smallest, m.smallest_at) = (2.0 * t.area / longest, ((t.p[0] + t.p[1] + t.p[2]) / 3.0).to_array());
        }
    }
    m.wall_area = tally.wall_area / (BAY * STOREY);
    m.roof_area = tally.roof_area / (BAY * BAY);
    m.wall = if m.wall_area > 0.0 { tally.wall_tris as f32 / m.wall_area } else { 0.0 };
    m.roof = if m.roof_area > 0.0 { tally.roof_tris as f32 / m.roof_area } else { 0.0 };
    let (bare, bare_at) = blank(&ts, ground);
    (m.blank, m.blank_at) = (bare / BAY, bare_at.to_array());
    m.storey = stacked(b, pal);
    m
}

/// How much upright face a mesh has above the height of a yard wall, in
/// bays of a storey: what a building's walls come to, however they were built.
pub fn upright(b: &MeshBuf) -> f32 {
    // (the thing stands on level ground at the origin)
    let ground = 0.0;
    tris(b).iter().filter(|t| t.n.y.abs() < UPRIGHT && t.p.iter().map(|p| p.y).fold(f32::MIN, f32::max) - ground > LOW).map(|t| t.area).sum::<f32>() / (BAY * STOREY)
}

/// The longest run of bare wall: over every big upright face, the longest
/// stretch along it that no other face stands on or before.
fn blank(ts: &[Tri], ground: f32) -> (f32, Vec3) {
    /// what stands this near before a wall is on it or against it
    const BEFORE: f32 = 0.012;
    let mut worst = 0.0f32;
    let mut at_worst = Vec3::ZERO;
    for (i, t) in ts.iter().enumerate() {
        if t.n.y.abs() >= UPRIGHT {
            continue;
        }
        let along = Vec3::new(-t.n.z, 0.0, t.n.x).normalize_or_zero();
        let u = t.p.map(|p| p.dot(along));
        let (u0, u1) = (u.iter().copied().fold(f32::MAX, f32::min), u.iter().copied().fold(f32::MIN, f32::max));
        let (y0, y1) = (t.p.iter().map(|p| p.y).fold(f32::MAX, f32::min), t.p.iter().map(|p| p.y).fold(f32::MIN, f32::max));
        // a wall of a building: long, and standing most of a storey high above the ground (a
        // strip between two rows of openings is no blank wall, nor is a yard wall or a low bank)
        if u1 - u0 <= worst.max(2.0 * BAY) || y1 - y0.max(ground) < 0.9 * STOREY {
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
        if gap > worst {
            (worst, at_worst) = (gap, (t.p[0] + t.p[1] + t.p[2]) / 3.0);
        }
    }
    (worst, at_worst)
}

/// How far apart windows of one size stand one above another: the middle
/// such distance, from the flat panes of glass on upright faces (which is
/// how the play level draws a window), or nothing if no two are stacked.
fn stacked(b: &MeshBuf, pal: &Palette) -> Option<f32> {
    let (glass, lit) = (*pal.glass.window, *pal.glass.mill_lit);
    // every upright quad of glass: its plane's facing, its middle along the wall, its foot, width and height
    let mut panes: Vec<(Vec3, f32, f32, f32, f32, f32)> = Vec::new();
    for q in b.idx.chunks(6) {
        if q.len() < 6 || q.iter().any(|&i| b.col[i as usize] != glass && b.col[i as usize] != lit) {
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
        out.push(format!("its smallest feature is {:.5} (at {:.3?}), finer than the {} level's {:.5}", m.smallest, m.smallest_at, detail.label(), lv.finest));
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
        out.push(format!("a wall is bare for {:.1} modules at {:.3?} (no more than {BLANK} may be)", m.blank, m.blank_at));
    }
    if let Some(s) = m.storey {
        let n = (s / STOREY).round().max(1.0);
        if (s - n * STOREY).abs() > 0.02 * STOREY {
            out.push(format!("its windows stand {s:.4} apart one above another: a storey is {STOREY}"));
        }
    }
    out
}

/// A measure in a line, for a label on the lab's `scale` sheet: the smallest feature in
/// thousandths of a map unit, the triangles to a bay of a storey of wall and to a bay
/// by a bay of roof, and the longest bare wall in bays.
pub fn words(m: &Measure) -> String {
    format!("finest {:.2}, wall {:.1}, roof {:.1}, bare {:.1}", m.smallest * 1000.0, m.wall, m.roof, m.blank)
}
