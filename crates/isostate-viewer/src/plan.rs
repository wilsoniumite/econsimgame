//! A top-down plan of the ground, drawn on the CPU from the same rules the
//! shader and the scattered things follow: land classes, hedges, rivers and
//! relief. For checking the map's geometry in seconds rather than waiting for a
//! render. Set ISOSTATE_PLAN to a path ending in .png or .ppm and the viewer
//! writes the plan there and stops; ISOSTATE_PLAN_VIEW=x0,z0,x1,z1,pixels-per-cell
//! picks the area (the whole map at 10 by default) and ISOSTATE_PLAN_YEAR the
//! year. ISOSTATE_PLAN_FIELD=NAME paints, in place of the land's classes, one
//! of the fields they are decided by as a ramp from dark blue (0) to pale
//! yellow (1): the ranks `wild`, `moor` and `lea` of `world::Land`, `wood` (how
//! wooded each parcel is this year), `ley`, `soil`, `depth` (how finely a
//! parcel is cut into fields) or `open` (the open fields). ISOSTATE_AREAS=1
//! measures how well the drawn land keeps the areas the record gives, and stops
//! (`maybe_areas`).

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

/// Write a picture of w by h pixels, three bytes each: a PNG if the path ends
/// in .png, a PPM otherwise.
pub fn write_picture(path: &str, w: usize, h: usize, rgb: &[u8]) -> Result<(), String> {
    if !path.ends_with(".png") {
        let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
        out.extend_from_slice(rgb);
        return std::fs::write(path, out).map_err(|e| e.to_string());
    }
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let rgba: Vec<u8> = rgb.chunks(3).flat_map(|c| [c[0], c[1], c[2], 255]).collect();
    let size = Extent3d { width: w as u32, height: h as u32, depth_or_array_layers: 1 };
    let img = bevy::image::Image::new(size, TextureDimension::D2, rgba, TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default());
    img.try_into_dynamic().map_err(|e| e.to_string())?.save(path).map_err(|e| e.to_string())
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
/// several times over, then stop. The houses are built for a camera over the
/// first town at the distance the game is played from; ISOSTATE_BENCH names
/// another: `close` (as near as the camera goes), `far` (half way out to the
/// whole island), or `near`, `play` or `shells` for every house of the island
/// at one level.
pub fn maybe_bench(r: &Replay, g: &Ground, slots: &Slots, cen: &[(f32, f32)]) {
    if std::env::var_os("ISOSTATE_BENCH").is_none() {
        return;
    }
    let years: Vec<usize> = [1760, 1800, 1850, 1870, 1900].iter().map(|y| ((y - r.y0).max(0) as usize).min(r.ny - 1)).collect();
    let lk = crate::look::Look::standard();
    let home = r.header.towns.first().map_or((r.w as f32 * 0.5, r.h as f32 * 0.5), |t| cen[t.node]);
    let which = std::env::var("ISOSTATE_BENCH").unwrap_or_default();
    let lod = match which.as_str() {
        "close" => crate::scatter::Lod::of_camera(home, crate::shots::CLOSE_DIST),
        "far" => crate::scatter::Lod::of_camera(home, crate::shots::FAR_DIST),
        "near" => crate::scatter::Lod::all(isostate_gen::houses::Detail::Near),
        "play" => crate::scatter::Lod::all(isostate_gen::houses::Detail::Play),
        "shells" => crate::scatter::Lod::all(isostate_gen::houses::Detail::Far),
        _ => crate::scatter::Lod::of_camera(home, crate::shots::PLAY_DIST),
    };
    eprintln!("bench: houses built for {lod:?}");
    let mut total = 0.0;
    let mut n = 0;
    for round in 0..3 {
        for &k in &years {
            let t0 = seconds();
            let ym = crate::scatter::build_year(r, g, slots, cen, k, &lk, &lod);
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

/// With ISOSTATE_AREAS set, measure how well the drawn land keeps the areas
/// the simulation gives, and stop. For a handful of years it adds up arable,
/// pasture, wood and heath as shares of the open land, over the whole island
/// and over every district of six squares by six (12 km) that is mostly land:
/// what the record says (the cells' mixes), what `world::year_land` draws (the
/// dry area of the parcels of each class), and what the patchwork rule cell
/// by cell (`world::pick_class`) would draw, to compare.
pub fn maybe_areas(r: &Replay, g: &Ground) {
    if std::env::var_os("ISOSTATE_AREAS").is_none() {
        return;
    }
    const D: usize = 6;
    let (dw, dh) = (r.w.div_ceil(D), r.h.div_ceil(D));
    let district = |x: f32, y: f32| ((y.max(0.0) as usize).min(r.h - 1) / D) * dw + (x.max(0.0) as usize).min(r.w - 1) / D;
    let l = &g.land;
    let names = ["arable", "pasture", "wood", "heath"];
    for year in [1750, 1800, 1850, 1900] {
        let k = ((year - r.y0).max(0) as usize).min(r.filled.clamp(1, r.ny) - 1);
        let land = world::year_land(r, g, k);
        // per district: the record's areas, the drawn, and the cell rule's
        let mut sums = vec![[[0.0f64; 4]; 3]; dw * dh];
        let mut cells = vec![0usize; dw * dh];
        for c in 0..r.n {
            if r.terrain[c] == 0 {
                continue;
            }
            let d = (c / r.w / D) * dw + (c % r.w) / D;
            cells[d] += 1;
            for (t, s) in world::open_shares(r, k, c).iter().enumerate() {
                sums[d][0][t] += *s as f64;
            }
        }
        for i in 0..l.at.len() {
            if l.area[i] <= 0.0 {
                continue;
            }
            let d = district(l.at[i].x, l.at[i].y);
            let old = if r.terrain[l.wcell[i] as usize] == 0 { world::SEA } else { world::pick_class(r, k, l.wcell[i] as usize, l.sub[i]) };
            for (which, cls) in [(1, land.class[i]), (2, old)] {
                if (world::FARM..=world::HEATH).contains(&cls) {
                    sums[d][which][(cls - world::FARM) as usize] += l.area[i] as f64;
                }
            }
        }
        let share = |v: &[f64; 4]| -> [f64; 4] {
            let t: f64 = v.iter().sum::<f64>().max(1e-9);
            v.map(|x| 100.0 * x / t)
        };
        let mut all = [[0.0f64; 4]; 3];
        for d in &sums {
            for w in 0..3 {
                for t in 0..4 {
                    all[w][t] += d[w][t];
                }
            }
        }
        let (rec, drawn, old) = (share(&all[0]), share(&all[1]), share(&all[2]));
        eprintln!("areas {}: open land of the whole island, % (record / drawn / cell rule):", r.y0 + k as i32);
        for t in 0..4 {
            eprintln!("  {:8} {:5.1} / {:5.1} / {:5.1}", names[t], rec[t], drawn[t], old[t]);
        }
        // the same as it is painted, with the edges of the woods and heaths leaning across
        // the parcels' sides: the class under ten points each way to a cell
        let grid = 10;
        let counts = (0..r.h * grid)
            .into_par_iter()
            .map(|j| {
                let mut n = [0.0f64; 5];
                for i in 0..r.w * grid {
                    let (x, z) = ((i as f32 + 0.5) / grid as f32, (j as f32 + 0.5) / grid as f32);
                    if world::height_at(g, x, z) > 0.02 {
                        let (cls, margin) = land.at(world::place_at(g, x, z));
                        if (world::FARM..=world::HEATH).contains(&cls) {
                            n[if margin { 4 } else { (cls - world::FARM) as usize }] += 1.0;
                        }
                    }
                }
                n
            })
            .reduce(|| [0.0; 5], |a, b| std::array::from_fn(|t| a[t] + b[t]));
        let all_painted: f64 = counts.iter().sum::<f64>().max(1.0);
        eprintln!("  as painted, with the edges of wood and heath leaning: {}, and their margins of rough grass {:.1}",
            (0..4).map(|t| format!("{} {:.1}", names[t], 100.0 * counts[t] / all_painted)).collect::<Vec<_>>().join(", "), 100.0 * counts[4] / all_painted);
        // the districts that are mostly land: how far each class's share is out, in points of the open land
        let (mut n, mut mean, mut worst) = (0, [0.0f64; 2], [0.0f64; 2]);
        let mut by_class = [[0.0f64; 4]; 2];
        for (d, s) in sums.iter().enumerate() {
            if cells[d] < D * D * 3 / 4 || s[0].iter().sum::<f64>() < 4.0 {
                continue;
            }
            n += 1;
            let rec = share(&s[0]);
            for (w, which) in [1, 2].into_iter().enumerate() {
                let got = share(&s[which]);
                for t in 0..4 {
                    let e = (got[t] - rec[t]).abs();
                    mean[w] += e / 4.0;
                    by_class[w][t] += e;
                    worst[w] = worst[w].max(e);
                }
            }
        }
        let nn = (n as f64).max(1.0);
        eprintln!("  over {n} districts of {D} by {D} squares, a class's share is out by {:.1} points on average and {:.1} at worst (the cell rule: {:.1} and {:.1})", mean[0] / nn, worst[0], mean[1] / nn, worst[1]);
        eprintln!("  by class, on average: {} (the cell rule: {})",
            (0..4).map(|t| format!("{} {:.1}", names[t], by_class[0][t] / nn)).collect::<Vec<_>>().join(", "),
            (0..4).map(|t| format!("{} {:.1}", names[t], by_class[1][t] / nn)).collect::<Vec<_>>().join(", "));
    }
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
    let land = world::year_land(r, g, k);
    let field = std::env::var("ISOSTATE_PLAN_FIELD").ok();
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
            } else if let Some(field) = &field {
                // one of the fields the lie of the land is decided by, as a ramp
                // from dark blue (0) through green to pale yellow (1)
                let i = world::parcel_index(g, x, z);
                let v = match field.as_str() {
                    "wild" => g.land.wild[i],
                    "moor" => g.land.moor[i],
                    "lea" => g.land.lea[i],
                    "wood" => land.wood[i] as f32 / 255.0,
                    "ley" => g.land.ley[i] as f32 / 255.0,
                    "soil" => g.land.soil[i] as f32 / 255.0,
                    "depth" => (g.land.depth[i] as f32 - 3.0) / 3.0,
                    _ => if g.land.open_until[i] > 0 { 1.0 } else { 0.0 },
                };
                [0.1 + 0.85 * v * v, 0.15 + 0.75 * (v * 1.6).min(1.0), 0.45 - 0.3 * v + 0.35 * (v - 0.7).max(0.0) * 3.0]
            } else {
                let cls = world::class_at(g, &land, x, z);
                let mut c = class_rgb(cls);
                if cls == world::FARM || cls == world::PASTURE {
                    let f = world::field_at(g, x, z);
                    let key = crate::fields::field_key(&f);
                    let v = 0.9 + 0.2 * crate::fields::hash3(key, 3, 5);
                    c = [c[0] * v, c[1] * v, c[2] * v];
                    // hedges, where a pixel is small enough to show a field
                    if px < 0.03 && f.edge < 0.7 * px + 0.006 {
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
        let cls = land.at(s.place).0;
        if px < 0.03 && (cls == world::FARM || cls == world::PASTURE) {
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
    let rgb: Vec<u8> = img.iter().flat_map(|c| c.map(|v| (v.clamp(0.0, 1.0) * 255.0) as u8)).collect();
    match write_picture(&path, w, h, &rgb) {
        Ok(()) => eprintln!("wrote the plan to {path}"),
        Err(e) => eprintln!("cannot write {path}: {e}"),
    }
    std::process::exit(0);
}
