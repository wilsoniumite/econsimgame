//! A top-down plan of the ground, drawn on the CPU from the same rules the
//! shader and the scattered things follow: land classes, hedges, rivers and
//! relief. For checking the map's geometry in seconds rather than waiting for a
//! render. Set ISOSTATE_PLAN to a path ending in .ppm and the viewer writes the
//! plan there and stops; ISOSTATE_PLAN_VIEW=x0,z0,x1,z1,pixels-per-cell picks the
//! area (the whole map at 10 by default) and ISOSTATE_PLAN_YEAR the year.

use rayon::prelude::*;

use crate::replay::Replay;
use crate::scatter::Slots;
use crate::world::{self, Ground};

fn class_rgb(c: u8) -> [f32; 3] {
    let h = match c {
        world::FARM => 0xd9c27a,
        world::PASTURE => 0x9cc463,
        world::WOOD => 0x4f7a3a,
        world::HEATH => 0xb3a37e,
        world::QUARRY => 0xc9c3b7,
        world::CLAYPIT => 0xc69466,
        world::IRONMINE => 0xa76c4c,
        world::COALMINE => 0x4a4643,
        world::WORKS => 0x8f857a,
        world::URBAN => 0xcbb9a1,
        _ => 0x968f81,
    };
    [((h >> 16) & 255) as f32 / 255.0, ((h >> 8) & 255) as f32 / 255.0, (h & 255) as f32 / 255.0]
}

/// Seconds since the first call, on the clock on the wall. This used to be
/// the thread's CPU time read from /proc, which exists on Linux only: on
/// Windows and macOS every timing came out as 0 ms. Time on the wall counts
/// whatever else the machine is doing, so bench on a quiet one.
fn seconds() -> f64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START.get_or_init(std::time::Instant::now).elapsed().as_secs_f64()
}

/// With ISOSTATE_BENCH set, time the rebuild of a handful of years (the year's
/// scattered things, its lines and its trains, and turning them into meshes),
/// several times over, then stop.
pub fn maybe_bench(r: &Replay, g: &Ground, slots: &Slots, cen: &[(f32, f32)]) {
    if std::env::var_os("ISOSTATE_BENCH").is_none() {
        return;
    }
    let years: Vec<usize> = [1760, 1800, 1850, 1870, 1900].iter().map(|y| ((y - r.y0).max(0) as usize).min(r.ny - 1)).collect();
    let lk = crate::look::Look::standard();
    let mut total = 0.0;
    let mut n = 0;
    for round in 0..3 {
        for &k in &years {
            let t0 = seconds();
            let ym = crate::scatter::build_year(r, g, slots, cen, k, &lk);
            let t1 = seconds();
            let nets = crate::network::year_nets(r, g, cen, k, &lk);
            let paths = crate::trains::rail_paths(r, g, cen, k);
            let t2 = seconds();
            let trees: usize = ym.trees.iter().flat_map(|t| t.parts()).map(|b| b.pos.len()).sum();
            let bare: usize = ym.trees.iter().map(|t| t.bare.pos.len()).sum();
            let verts = trees + ym.houses.pos.len() + ym.works.pos.len() + ym.smoke.pos.len() + ym.lanes.pos.len() + nets.roads.pos.len() + nets.rails.pos.len();
            if round == 0 {
                eprintln!("bench {} vertices: trees {} (of them bare limbs {}), houses {}, works {}, smoke {}, lanes {}, roads {}, rails {}", r.y0 + k as i32,
                    trees, bare, ym.houses.pos.len(), ym.works.pos.len(), ym.smoke.pos.len(), ym.lanes.pos.len(), nets.roads.pos.len(), nets.rails.pos.len());
            }
            let tree_meshes: Vec<_> = ym.trees.into_iter().flat_map(|t| [t.evergreen, t.leaves, t.bare]).map(crate::meshkit::into_mesh).collect();
            let meshes = [ym.houses, ym.works, ym.smoke, ym.lanes, nets.roads, nets.rails].map(crate::meshkit::into_mesh);
            let t3 = seconds();
            drop((meshes, tree_meshes));
            if round > 0 {
                total += t3 - t0;
                n += 1;
            }
            eprintln!("bench {} round {round}: scatter {:.0} ms, lines and trains {:.0} ms, meshes {:.0} ms; {} vertices, {} train paths",
                r.y0 + k as i32, (t1 - t0) * 1000.0, (t2 - t1) * 1000.0, (t3 - t2) * 1000.0, verts, paths.len());
        }
    }
    eprintln!("bench: {:.0} ms per year on average, by the clock on the wall", total / n as f64 * 1000.0);
    std::process::exit(0);
}

pub fn maybe_write(r: &Replay, g: &Ground, slots: &Slots) {
    let Ok(path) = std::env::var("ISOSTATE_PLAN") else { return };
    let view: Vec<f32> = std::env::var("ISOSTATE_PLAN_VIEW")
        .unwrap_or_default()
        .split(',')
        .filter_map(|s| s.parse().ok())
        .collect();
    let (x0, z0, x1, z1, ppc) = if view.len() == 5 { (view[0], view[1], view[2], view[3], view[4]) } else { (0.0, 0.0, r.w as f32, r.h as f32, 10.0) };
    let k = std::env::var("ISOSTATE_PLAN_YEAR")
        .ok()
        .and_then(|y| y.parse::<i32>().ok())
        .map(|y| ((y - r.y0).max(0) as usize).min(r.ny - 1))
        .unwrap_or(r.ny - 1);
    let (w, h) = (((x1 - x0) * ppc) as usize, ((z1 - z0) * ppc) as usize);
    let px = 1.0 / ppc;
    let mut img: Vec<[f32; 3]> = (0..w * h)
        .into_par_iter()
        .map(|i| {
            let (x, z) = (x0 + ((i % w) as f32 + 0.5) * px, z0 + ((i / w) as f32 + 0.5) * px);
            let hh = world::height_at(g, x, z);
            // light from the south-west, as in the scene
            let e = 0.02;
            let dx = (world::height_at(g, x + e, z) - world::height_at(g, x - e, z)) / (2.0 * e);
            let dz = (world::height_at(g, x, z + e) - world::height_at(g, x, z - e)) / (2.0 * e);
            let n = bevy::math::Vec3::new(-dx, 1.0, -dz).normalize();
            let sun = bevy::math::Vec3::new(-0.55, 0.62, 0.55).normalize();
            let lit = (0.45 + 0.75 * n.dot(sun).max(0.0)).min(1.25);
            let mut c = if hh < 0.0 {
                let d = (-hh / 0.4).min(1.0);
                [0.42 - 0.2 * d, 0.66 - 0.25 * d, 0.72 - 0.15 * d]
            } else if hh < 0.02 && world::flood_at(g, x, z) < 0.5 {
                [0.88, 0.82, 0.62]
            } else {
                let (cls, _) = world::class_at(r, g, k, x, z);
                let mut c = class_rgb(cls);
                if cls == world::FARM || cls == world::PASTURE {
                    let f = world::field_at(g, x, z);
                    let key = crate::fields::field_key(&f);
                    let v = 0.9 + 0.2 * crate::fields::hash3(key, 3, 5);
                    c = [c[0] * v, c[1] * v, c[2] * v];
                    if f.edge < 0.7 * px + 0.006 {
                        c = [0.25, 0.42, 0.2];
                    }
                }
                c
            };
            if hh >= 0.0 {
                c = [c[0] * lit, c[1] * lit, c[2] * lit];
            }
            if world::water_dist(g, x, z) < 0.0 {
                c = [0.36, 0.62, 0.78];
            }
            c
        })
        .collect();
    // the scattered trees, as dark dots
    let mut dot = |x: f32, z: f32, col: [f32; 3]| {
        let (i, j) = (((x - x0) * ppc) as isize, ((z - z0) * ppc) as isize);
        for dj in -1..=1isize {
            for di in -1..=1isize {
                let (a, b) = (i + di, j + dj);
                if a >= 0 && b >= 0 && (a as usize) < w && (b as usize) < h {
                    img[b as usize * w + a as usize] = col;
                }
            }
        }
    };
    for s in &slots.hedge {
        let cls = world::pick_class(r, k, s.wcell as usize, s.sub);
        if cls == world::FARM || cls == world::PASTURE {
            dot(s.x, s.z, [0.12, 0.3, 0.1]);
        }
    }
    // where the railway's works are, for pointing a camera at them
    let ry = crate::rails::year(r, g, &crate::network::centres(r), k);
    for (li, line) in ry.lines.iter().enumerate() {
        let mut i = 0;
        while i < line.kind.len() {
            let kind = line.kind[i];
            let mut j = i;
            while j + 1 < line.kind.len() && line.kind[j + 1] == kind {
                j += 1;
            }
            if kind != crate::rails::Kind::Grade && j - i >= 3 {
                let (a, b) = (line.pts[i], line.pts[j]);
                eprintln!("line {li}: {:?} from {:.1},{:.1} to {:.1},{:.1}", kind, a.x, a.z, b.x, b.z);
            }
            i = j + 1;
        }
    }
    let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
    for c in img {
        for v in c {
            out.push((v.clamp(0.0, 1.0) * 255.0) as u8);
        }
    }
    match std::fs::write(&path, out) {
        Ok(()) => eprintln!("wrote the plan to {path}"),
        Err(e) => eprintln!("cannot write {path}: {e}"),
    }
    std::process::exit(0);
}
