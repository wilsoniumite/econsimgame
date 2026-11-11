//! The standard shots: a fixed, named set of views of one game, for looking at
//! a change to the picture before and after.
//!
//!     isostate-viewer --shots out/before
//!     isostate-viewer --shots out/after --shots-compare out/before
//!     isostate-viewer --compare-shots out/before out/after
//!
//! The views are chosen from the simulation (the player's home town, the cell
//! with the most works), so they follow the game on any seed. Each set gets an
//! `index.html`, a contact sheet of the shots with their names; with another
//! set to compare against, every shot stands beside the one of the same name
//! and the sheet says how far the two differ, pixel by pixel.
//!
//! While the shots are taken the clock that moves trains, carts, boats and the
//! sea stands still, so two runs of the same code give the same pictures.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, Image, ImageSampler, ImageType};
use bevy::math::{Vec2, Vec3};
use bevy::prelude::Resource;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::replay::{self, Replay};
use crate::{network, scatter, ShotSpec};

/// The closest the camera comes to what it looks at.
pub const CLOSE_DIST: f32 = 1.5;
/// The distance a new game opens at, over the home town.
pub const PLAY_DIST: f32 = 14.0;
/// The furthest the camera goes.
pub const FURTHEST_DIST: f32 = 220.0;
/// About half way out from play distance to the whole island, which fills a
/// picture of 1600 by 1000 from about 134 on island 7 (`island_view`).
pub const FAR_DIST: f32 = 70.0;
/// The tilt a game opens at, in degrees above the horizon.
pub const PLAY_PITCH: f32 = 52.0;
/// The angle the whole island is seen from, as a replay opens.
pub const ISLAND_PITCH: f32 = 52.0;

/// Where to look, and from how far, for the whole island to stand in a picture
/// of this size, clear of the cards along the top and the timeline below. (A
/// replay's own opening view is closer and cuts the south coast off.)
pub fn island_view(r: &Replay, size: (u32, u32)) -> ((f32, f32), f32) {
    // the corners of every land cell on the coast, at sea level
    let land = |x: isize, z: isize| x >= 0 && z >= 0 && (x as usize) < r.w && (z as usize) < r.h && r.terrain[z as usize * r.w + x as usize] > 0;
    let mut pts: Vec<Vec3> = Vec::new();
    for c in 0..r.n {
        let (x, z) = ((c % r.w) as isize, (c / r.w) as isize);
        if land(x, z) && !(land(x - 1, z) && land(x + 1, z) && land(x, z - 1) && land(x, z + 1)) {
            for (dx, dz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                pts.push(Vec3::new((x + dx) as f32, 0.0, (z + dz) as f32));
            }
        }
    }
    if pts.is_empty() {
        return ((r.w as f32 * 0.5, r.h as f32 * 0.5), r.w.max(r.h) as f32 * 1.6);
    }
    let (x0, x1) = pts.iter().fold((f32::MAX, f32::MIN), |a, p| (a.0.min(p.x), a.1.max(p.x)));
    let (z0, z1) = pts.iter().fold((f32::MAX, f32::MIN), |a, p| (a.0.min(p.z), a.1.max(p.z)));
    let (w, h) = (size.0 as f32, size.1 as f32);
    // the part of the picture left free by the interface, in the camera's own
    // terms (-1 to 1 each way): 88 pixels are taken above and 76 below
    let (top, bottom, side) = (1.0 - 2.0 * 88.0 / h, -1.0 + 2.0 * 76.0 / h, 1.0 - 2.0 * 14.0 / w);
    let tan = (crate::FOV_DEG.to_radians() * 0.5).tan();
    let pitch = ISLAND_PITCH.to_radians();
    // the camera stands south of the point it looks at
    let back = Vec3::new(0.0, pitch.sin(), pitch.cos());
    let up = Vec3::new(0.0, pitch.cos(), -pitch.sin());
    let fits = |tz: f32, dist: f32| {
        let cam = Vec3::new((x0 + x1) * 0.5, 0.0, tz) + back * dist;
        pts.iter().all(|p| {
            let v = *p - cam;
            let depth = -v.dot(back);
            let (sx, sy) = (v.x / (depth * tan * w / h), v.dot(up) / (depth * tan));
            depth > 0.0 && sx.abs() <= side && sy <= top && sy >= bottom
        })
    };
    // the nearest distance at which some point to look at fits it all in
    let steps = || (0..=200).map(|i| z0 + (z1 - z0) * i as f32 / 200.0);
    let (mut near, mut far) = (1.0f32, 4.0 * (x1 - x0).max(z1 - z0) + 50.0);
    for _ in 0..24 {
        let mid = (near + far) * 0.5;
        if steps().any(|tz| fits(tz, mid)) { far = mid } else { near = mid }
    }
    // of the points that fit at that distance, the middle one, so the island sits in the middle
    let ok: Vec<f32> = steps().filter(|&tz| fits(tz, far)).collect();
    let tz = if ok.is_empty() { (z0 + z1) * 0.5 } else { (ok[0] + ok[ok.len() - 1]) * 0.5 };
    (((x0 + x1) * 0.5, tz), far)
}

/// One shot of the set: where it is written, and what the sheet says about it.
#[derive(Clone)]
pub struct Standard {
    pub name: String,
    pub caption: String,
    pub spec: ShotSpec,
}

/// What the sheet says about the whole set.
#[derive(Clone, Default)]
pub struct About {
    /// one line on what was shot: the seed, the families, the size
    pub what: String,
    /// a note from the command line (`--shots-note`), such as the commit
    pub note: String,
}

/// The set being shot, kept until the last picture is written.
#[derive(Resource, Clone)]
pub struct Sheet {
    pub dir: String,
    pub compare: Option<String>,
    pub shots: Vec<(String, String)>,
    pub about: About,
}

/// Remove the pictures a set is about to write. A set shot into a directory
/// that already holds one would otherwise find last time's pictures there,
/// take them for its own before they were written again, and compare those.
pub fn clear_pictures(shots: &[Standard]) {
    for s in shots {
        let _ = std::fs::remove_file(&s.spec.path);
    }
}

/// The standard views, in the order the sheet shows them. `home` is the town
/// the player's family sits in (the biggest town, in a replay); `island` is the
/// point and distance the whole island is seen from.
pub fn standard_views(r: &Replay, cen: &[(f32, f32)], slots: &scatter::Slots, home: Option<usize>, island: ((f32, f32), f32), dir: &str) -> Vec<Standard> {
    let last = r.filled.clamp(1, r.ny) - 1;
    let year_of = |y: i32| y.clamp(r.y0, r.y0 + last as i32);
    let k_of = |y: i32| (year_of(y) - r.y0) as usize;
    let path = |name: &str| Path::new(dir).join(format!("{name}.png")).to_string_lossy().replace('\\', "/");
    let mut out: Vec<Standard> = Vec::new();
    let mut add = |name: String, caption: String, year: i32, look: (f32, f32), yaw: f32, pitch: f32, dist: f32, season: f32| {
        out.push(Standard {
            spec: ShotSpec { path: path(&name), year: Some(year_of(year)), look: Some(look), yaw, pitch, dist: Some(dist), season: Some(season), tab: Some(None), overlay: None, markets: None },
            name,
            caption,
        });
    };
    const SUMMER: f32 = 0.5;
    const MIDWINTER: f32 = 0.0;

    for y in [1750, 1800, 1850, 1900] {
        add(format!("island-{y}"), format!("The whole island, summer {}", year_of(y)), y, island.0, 0.0, ISLAND_PITCH, island.1, SUMMER);
    }

    // the home town: the player's seat, or the biggest town there is at the end
    let town = home.or_else(|| {
        (0..r.n_towns.min(r.header.towns.len()))
            .filter(|&t| r.town_active[last * r.n_towns + t] > 0)
            .max_by(|&a, &b| r.town_urban[last * r.n_towns + a].total_cmp(&r.town_urban[last * r.n_towns + b]))
    });
    if let Some(t) = town {
        let name = r.header.towns[t].name.clone();
        let node = r.header.towns[t].node;
        let (x, z) = cen[node];
        // as a new game opens: the town a little above the middle of the picture
        let play = (x, z + 1.0);
        for y in [1750, 1800, 1850, 1900] {
            add(format!("town-{y}"), format!("{name}, the home town, at play distance, summer {}", year_of(y)), y, play, 0.0, PLAY_PITCH, PLAY_DIST, SUMMER);
        }
        add("town-1850-winter".into(), format!("{name} at play distance, midwinter {}", year_of(1850)), 1850, play, 0.0, PLAY_PITCH, PLAY_DIST, MIDWINTER);
        // a street: a little way out of the market place along the first road
        // through town, seen from the side the sun is on
        let k = k_of(1850);
        let along = network::road_segments(r, k)
            .into_iter()
            .find_map(|(c, m, _)| {
                let other = if c == node { m } else if m == node { c } else { return None };
                Some((Vec2::new(cen[other].0, cen[other].1) - Vec2::new(x, z)).normalize_or(Vec2::X))
            })
            .unwrap_or(Vec2::X);
        let at = Vec2::new(x, z) + along * 0.16;
        let sun = Vec2::new(-0.682, 0.731);
        let across = Vec2::new(-along.y, along.x);
        let across = if across.dot(sun) >= 0.0 { across } else { -across };
        let from = (across + along * 0.7).normalize();
        add("street-1850".into(), format!("A street in {name}, close up, summer {}", year_of(1850)), 1850, (at.x, at.y), from.x.atan2(from.y).to_degrees(), 30.0, 1.9, SUMMER);
    }

    // the busiest works district at the end: the cell with the most works standing
    let mut count = vec![0u32; r.n];
    let mut sum = vec![(0f32, 0f32); r.n];
    for (f, firm) in r.header.firms.iter().enumerate() {
        if replay::firm_k(r, f, k_of(1900)) > 0.0 && r.header.methods[firm.method].name != "services" && firm.cell < r.n {
            if let Some(&(px, pz, _)) = slots.firm_pos.get(f) {
                count[firm.cell] += 1;
                sum[firm.cell].0 += px;
                sum[firm.cell].1 += pz;
            }
        }
    }
    // the first of the cells with the most, so the choice is the same every run
    if let Some(c) = (0..r.n).rev().max_by_key(|&c| count[c]).filter(|&c| count[c] > 0) {
        let n = count[c] as f32;
        let at = (sum[c].0 / n, sum[c].1 / n);
        let place = format!("the square {} km east, {} km south ({} works)", (c % r.w) * 2, (c / r.w) * 2, count[c]);
        add("works-1900".into(), format!("The busiest works district at play distance, summer {}: {place}", year_of(1900)), 1900, (at.0, at.1 + 1.0), 0.0, PLAY_PITCH, PLAY_DIST, SUMMER);
        add("works-1900-close".into(), format!("The busiest works district close up, summer {}: {place}", year_of(1900)), 1900, at, 35.0, 32.0, 3.2, SUMMER);
    }
    out
}

/// The far views of the land (`--land-shots DIR`): five cameras, each in 1800
/// and 1900 in summer, in 1800 in October and in 1900 in midwinter, for
/// judging a change to the rules the land is laid out and painted by
/// (`world::year_land`, the land blocks of terrain.wesl). Like the standard
/// views they are chosen from the run, so they follow the game on any island:
/// the whole island; from 8 km up, the lowland where wood and field are most
/// mixed in 1800; the hill country where pasture and heath are most mixed in
/// 1900; the country round the home town; and the highest ground, for the
/// snow line.
pub fn land_views(r: &Replay, g: &crate::world::Ground, cen: &[(f32, f32)], home: Option<usize>, island: ((f32, f32), f32), dir: &str) -> Vec<Standard> {
    use crate::world;
    let last = r.filled.clamp(1, r.ny) - 1;
    let year_of = |y: i32| y.clamp(r.y0, r.y0 + last as i32);
    let k_of = |y: i32| (year_of(y) - r.y0) as usize;
    let path = |name: &str| Path::new(dir).join(format!("{name}.png")).to_string_lossy().replace('\\', "/");
    // the middle of the window of `half` cells each way that scores highest, a cell's score
    // being None where the window may not reach (the sea, the wrong kind of country)
    let best = |half: isize, score: &dyn Fn(usize) -> Option<f32>| -> Option<(f32, f32)> {
        let cell: Vec<Option<f32>> = (0..r.n).map(|c| if r.terrain[c] == 0 { None } else { score(c) }).collect();
        let mut top: Option<(f32, usize)> = None;
        for c in 0..r.n {
            let (x, z) = ((c % r.w) as isize, (c / r.w) as isize);
            let mut sum = 0.0;
            for dz in -half..=half {
                for dx in -half..=half {
                    let (xx, zz) = (x + dx, z + dz);
                    if xx >= 0 && zz >= 0 && (xx as usize) < r.w && (zz as usize) < r.h {
                        sum += cell[zz as usize * r.w + xx as usize].unwrap_or(0.0);
                    }
                }
            }
            if top.is_none_or(|t| sum > t.0) {
                top = Some((sum, c));
            }
        }
        top.filter(|t| t.0 > 0.0).map(|(_, c)| ((c % r.w) as f32 + 0.5, (c / r.w) as f32 + 0.5))
    };
    let mut cams: Vec<(&str, String, (f32, f32), f32, f32)> = vec![("island", "The whole island".into(), island.0, ISLAND_PITCH, island.1)];
    let k = k_of(1800);
    if let Some(at) = best(5, &|c| (r.terrain[c] == 1).then(|| { let s = world::open_shares(r, k, c); s[2].min(s[0] + s[1] + s[3]) })) {
        cams.push(("lowland", "Wooded lowland from 8 km up".into(), at, PLAY_PITCH, 40.0));
    }
    let k = k_of(1900);
    if let Some(at) = best(3, &|c| (r.terrain[c] == 2).then(|| { let s = world::open_shares(r, k, c); s[1].min(s[3]) })) {
        cams.push(("hills", "A hill margin, pasture against the moor".into(), at, PLAY_PITCH, 25.0));
    }
    let town = home.or_else(|| {
        (0..r.n_towns.min(r.header.towns.len()))
            .filter(|&t| r.town_active[last * r.n_towns + t] > 0)
            .max_by(|&a, &b| r.town_urban[last * r.n_towns + a].total_cmp(&r.town_urban[last * r.n_towns + b]))
    });
    if let Some(t) = town {
        let (x, z) = cen[r.header.towns[t].node];
        cams.push(("town", format!("The country round {}", r.header.towns[t].name), (x, z + 1.0), PLAY_PITCH, 24.0));
    }
    // the highest ground, if any of it keeps its snow through the summer
    let top = (0..g.gw * g.gh).max_by(|&a, &b| g.hgt[a].total_cmp(&g.hgt[b])).unwrap_or(0);
    if g.hgt[top] > 1.75 {
        cams.push(("snowline", "The snow line".into(), ((top % g.gw) as f32 / world::VRES as f32, (top / g.gw) as f32 / world::VRES as f32 + 1.0), PLAY_PITCH, 18.0));
    }
    let mut out = Vec::new();
    for (year, season, when) in [(1800, 0.5, "summer"), (1900, 0.5, "summer"), (1800, 0.8, "october"), (1900, 0.0, "midwinter")] {
        for (name, caption, look, pitch, dist) in &cams {
            let name = if when == "summer" { format!("{name}-{year}") } else { format!("{name}-{year}-{when}") };
            out.push(Standard {
                spec: ShotSpec { path: path(&name), year: Some(year_of(year)), look: Some(*look), yaw: 0.0, pitch: *pitch, dist: Some(*dist), season: Some(season), tab: Some(None), overlay: None, markets: None },
                name,
                caption: format!("{caption}, {when} {}", year_of(year)),
            });
        }
    }
    out
}

// ------------------------------------------------------------------ comparing two sets

/// How far two shots of the same name differ.
pub struct Diff {
    /// the largest difference in any channel of any pixel, of 255
    pub max: u8,
    /// the mean difference over every channel of every pixel, of 255
    pub mean: f64,
    /// how many pixels differ at all, and their share of the picture
    pub pixels: usize,
    pub changed: f64,
    /// the share of pixels that differ by more than 8 of 255 in some channel, enough to see
    pub visible: f64,
}

fn load_rgba(path: &Path) -> Result<(u32, u32, Vec<u8>), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let img = Image::from_buffer(&bytes, ImageType::Extension("png"), CompressedImageFormats::NONE, true, ImageSampler::Default, RenderAssetUsages::default())
        .map_err(|e| format!("cannot decode {}: {e}", path.display()))?;
    if img.texture_descriptor.format != TextureFormat::Rgba8UnormSrgb {
        return Err(format!("{} is not an 8-bit picture", path.display()));
    }
    let (w, h) = (img.width(), img.height());
    let data = img.data.ok_or_else(|| format!("{} has no pixels", path.display()))?;
    Ok((w, h, data))
}

/// Compare a `--ground-debug` shot (what the shader draws, in the flat class
/// colours of `debug_colour` in terrain.wesl) with the `--class-map` of the
/// same view (what `world::class_at` says, in `shot.classes.pgm` beside it).
/// It writes `shot.diff.png`: grey where the two agree, red where they differ
/// on a class edge (a pixel's middle can fall either side), yellow where they
/// differ away from one, dark where there is nothing to compare. It gives a
/// line saying how many pixels agree, and whether none differ away from an
/// edge but a few (under a thousandth of those compared).
pub fn compare_classes(shot: &Path) -> Result<(String, bool), String> {
    const PAL: [[i32; 3]; 12] =
        [[0, 0, 255], [255, 255, 0], [0, 255, 0], [0, 128, 0], [255, 0, 255], [0, 255, 255], [255, 128, 0], [128, 0, 0], [128, 0, 255], [255, 0, 0], [0, 128, 255], [128, 255, 128]];
    let (w, h, px) = load_rgba(shot)?;
    let (w, h) = (w as usize, h as usize);
    let pgm = shot.with_extension("classes.pgm");
    let data = std::fs::read(&pgm).map_err(|e| format!("cannot read {}: {e}", pgm.display()))?;
    // the header is three lines: P5, the size, 255
    let start = data.iter().enumerate().filter(|x| *x.1 == b'\n').nth(2).map(|(i, _)| i + 1).ok_or("the class map has no header")?;
    let cpu = &data[start..];
    if cpu.len() != w * h {
        return Err(format!("{} and {} are not the same size", shot.display(), pgm.display()));
    }
    // the shader's class under each pixel, where the pixel is a class colour
    let gpu: Vec<u8> = (0..w * h)
        .map(|i| {
            let p = &px[i * 4..i * 4 + 3];
            let d = |c: &[i32; 3]| (0..3).map(|k| (p[k] as i32 - c[k]).pow(2)).sum::<i32>();
            let best = (0..12).min_by_key(|&c| d(&PAL[c])).unwrap();
            if d(&PAL[best]) < 400 { best as u8 } else { 255 }
        })
        .collect();
    let (mut both, mut agree, mut differ, mut inner, mut unknown) = (0usize, 0usize, 0usize, 0usize, 0usize);
    let mut out = vec![0u8; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if cpu[i] == 255 {
                continue;
            }
            if gpu[i] == 255 {
                unknown += 1;
                continue;
            }
            both += 1;
            let col = if gpu[i] == cpu[i] {
                agree += 1;
                [150, 150, 150]
            } else {
                differ += 1;
                // next to a class edge of the CPU's map?
                let edge = (-1..=1isize).any(|dy| (-1..=1isize).any(|dx| {
                    let (xx, yy) = ((x as isize + dx).rem_euclid(w as isize) as usize, (y as isize + dy).rem_euclid(h as isize) as usize);
                    cpu[yy * w + xx] != cpu[i]
                }));
                if edge {
                    [255, 40, 40]
                } else {
                    inner += 1;
                    [255, 255, 0]
                }
            };
            out[i * 3..i * 3 + 3].copy_from_slice(&col);
        }
    }
    let diff = shot.with_extension("diff.png");
    crate::plan::write_picture(&diff.to_string_lossy(), w, h, &out)?;
    let line = format!(
        "{}: {both} land pixels compared, {agree} agree ({:.3}%), {differ} differ, of which {inner} away from a class edge; {unknown} land pixels without a class colour (interface, smoothing)",
        shot.display(),
        100.0 * agree as f64 / both.max(1) as f64
    );
    Ok((line, inner * 1000 <= both))
}

/// Compare two pictures pixel by pixel. Where they differ at all, a picture of
/// the difference (sixteen times over, so that one step of 255 can be seen) is
/// written to `diff_path`.
pub fn compare(a: &Path, b: &Path, diff_path: &Path) -> Result<Diff, String> {
    let (wa, ha, pa) = load_rgba(a)?;
    let (wb, hb, pb) = load_rgba(b)?;
    if (wa, ha) != (wb, hb) {
        return Err(format!("the sizes differ: {wa}x{ha} and {wb}x{hb}"));
    }
    let n = (wa * ha) as usize;
    let (mut max, mut sum, mut changed, mut visible) = (0u8, 0u64, 0usize, 0usize);
    let mut out = vec![255u8; n * 4];
    for i in 0..n {
        let mut worst = 0u8;
        for c in 0..3 {
            let d = pa[i * 4 + c].abs_diff(pb[i * 4 + c]);
            sum += d as u64;
            worst = worst.max(d);
            out[i * 4 + c] = d.saturating_mul(16);
        }
        max = max.max(worst);
        changed += (worst > 0) as usize;
        visible += (worst > 8) as usize;
    }
    if max > 0 {
        if let Some(parent) = diff_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let img = Image::new(Extent3d { width: wa, height: ha, depth_or_array_layers: 1 }, TextureDimension::D2, out, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
        match img.try_into_dynamic() {
            Ok(d) => {
                if let Err(e) = d.to_rgb8().save(diff_path) {
                    eprintln!("cannot write {}: {e}", diff_path.display());
                }
            }
            Err(e) => eprintln!("cannot write {}: {e}", diff_path.display()),
        }
    } else {
        // a picture of an older difference would mislead
        let _ = std::fs::remove_file(diff_path);
    }
    Ok(Diff { max, mean: sum as f64 / (n as f64 * 3.0), pixels: changed, changed: changed as f64 / n as f64, visible: visible as f64 / n as f64 })
}

// ------------------------------------------------------------------ the contact sheet

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// The way from one directory to another as a relative web path ("../before"),
/// or a file address where there is none (another drive).
fn relative(from: &Path, to: &Path) -> String {
    let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let (from, to) = (canon(from), canon(to));
    let (f, t): (Vec<_>, Vec<_>) = (from.components().collect(), to.components().collect());
    let common = f.iter().zip(&t).take_while(|(a, b)| a == b).count();
    if common == 0 {
        let s = to.to_string_lossy().replace('\\', "/");
        return format!("file:///{}", s.trim_start_matches("//?/").trim_start_matches('/'));
    }
    let mut parts: Vec<String> = std::iter::repeat_n("..".to_string(), f.len() - common).collect();
    parts.extend(t[common..].iter().map(|c| c.as_os_str().to_string_lossy().into_owned()));
    if parts.is_empty() { ".".into() } else { parts.join("/") }
}

fn dir_label(p: &Path) -> String {
    std::fs::canonicalize(p).ok().and_then(|c| c.file_name().map(|n| n.to_string_lossy().into_owned())).unwrap_or_else(|| p.to_string_lossy().into_owned())
}

const STYLE: &str = r#"
:root { --paper: #f6f0e2; --card: #fdf9ee; --ink: #332b24; --muted: #786b5c; --rule: #e3d9c4; --accent: #b23a3a; }
* { box-sizing: border-box; }
body { margin: 0; padding: 28px 32px 60px; background: var(--paper); color: var(--ink);
  font: 15px/1.45 "Fredoka", ui-rounded, "Segoe UI", system-ui, sans-serif; }
h1 { font-size: 26px; font-weight: 500; margin: 0 0 2px; }
p.about { margin: 0; color: var(--muted); }
table { border-collapse: collapse; margin: 18px 0 6px; background: var(--card); border-radius: 12px; overflow: hidden;
  box-shadow: 0 2px 8px rgba(64, 46, 26, 0.12); }
th, td { padding: 5px 14px; text-align: right; font-variant-numeric: tabular-nums; }
th { font-weight: 500; color: var(--muted); border-bottom: 1px solid var(--rule); }
th:first-child, td:first-child { text-align: left; }
a { color: inherit; text-decoration-color: var(--rule); text-underline-offset: 3px; }
tr.differs td { color: var(--accent); }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(560px, 1fr)); gap: 22px; margin-top: 22px; }
.grid.pairs { grid-template-columns: 1fr; }
figure { margin: 0; padding: 12px 12px 10px; background: var(--card); border-radius: 16px;
  box-shadow: 0 3px 10px rgba(64, 46, 26, 0.14); }
figcaption { display: flex; flex-wrap: wrap; gap: 2px 14px; align-items: baseline; margin: 0 4px 8px; }
figcaption b { font-weight: 500; font-size: 17px; }
figcaption span { color: var(--muted); }
figcaption .num { margin-left: auto; font-variant-numeric: tabular-nums; }
figcaption .num.differs { color: var(--accent); }
img { display: block; width: 100%; height: auto; border-radius: 9px; }
.pair { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
.pair a { display: block; color: var(--muted); text-decoration: none; text-align: center; font-size: 13px; }
.pair .tag { display: block; padding-top: 4px; }
.flip .stack { position: relative; display: block; }
.flip .under { position: absolute; inset: 0; }
.flip .over { position: relative; }
.flip:hover .over { opacity: 0; }
.missing { display: grid; place-items: center; min-height: 160px; border: 1px dashed var(--rule); border-radius: 9px; color: var(--muted); }
@media (max-width: 900px) { .pair { grid-template-columns: 1fr; } .grid { grid-template-columns: 1fr; } }
"#;

/// Write the sheet's list of shots beside the pictures, so that a set can be
/// compared later without shooting it again.
fn write_list(sheet: &Sheet) {
    let mut s = format!("# {}\n# {}\n", sheet.about.what, sheet.about.note);
    for (name, caption) in &sheet.shots {
        let _ = writeln!(s, "{name}\t{caption}");
    }
    let path = Path::new(&sheet.dir).join("shots.txt");
    if let Err(e) = std::fs::write(&path, s) {
        eprintln!("cannot write {}: {e}", path.display());
    }
}

/// Read a set back from its directory: the list written with it, or else every
/// PNG there by name.
fn read_sheet(dir: &str) -> Sheet {
    let mut sheet = Sheet { dir: dir.to_string(), compare: None, shots: Vec::new(), about: About::default() };
    if let Ok(text) = std::fs::read_to_string(Path::new(dir).join("shots.txt")) {
        let mut notes = text.lines().filter_map(|l| l.strip_prefix("# "));
        sheet.about.what = notes.next().unwrap_or("").to_string();
        sheet.about.note = notes.next().unwrap_or("").to_string();
        sheet.shots = text.lines().filter(|l| !l.starts_with('#')).filter_map(|l| l.split_once('\t')).map(|(n, c)| (n.to_string(), c.to_string())).collect();
    }
    if sheet.shots.is_empty() {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| e.file_name().to_string_lossy().strip_suffix(".png").map(String::from))
            .collect();
        names.sort();
        sheet.shots = names.into_iter().map(|n| (n, String::new())).collect();
    }
    sheet
}

/// Write `index.html` (and the list of shots) for a finished set, comparing it
/// with another set if one is named. Returns what was said about each shot
/// that was compared.
pub fn write_sheet(sheet: &Sheet) -> Vec<(String, Result<Diff, String>)> {
    let dir = Path::new(&sheet.dir);
    write_list(sheet);
    let other: Option<PathBuf> = sheet.compare.as_ref().map(PathBuf::from);
    let mut diffs: Vec<(String, Result<Diff, String>)> = Vec::new();
    if let Some(o) = &other {
        for (name, _) in &sheet.shots {
            let file = format!("{name}.png");
            let (a, b) = (o.join(&file), dir.join(&file));
            let d = if !a.exists() {
                Err(format!("no shot of this name in {}", dir_label(o)))
            } else if !b.exists() {
                Err("this shot was not written".to_string())
            } else {
                compare(&a, &b, &dir.join("diff").join(&file))
            };
            diffs.push((name.clone(), d));
        }
    }

    let title = match &other {
        Some(o) => format!("{} beside {}", dir_label(dir), dir_label(o)),
        None => dir_label(dir),
    };
    let mut h = String::new();
    let _ = write!(h, "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>Isolated State: {}</title>\n<style>{STYLE}</style>\n</head>\n<body>\n", esc(&title));
    let _ = writeln!(h, "<h1>Standard shots: {}</h1>", esc(&title));
    for line in [&sheet.about.what, &sheet.about.note] {
        if !line.is_empty() {
            let _ = writeln!(h, "<p class=\"about\">{}</p>", esc(line));
        }
    }
    if let Some(o) = &other {
        let rel = relative(dir, o);
        let _ = writeln!(h, "<p class=\"about\">Each shot stands beside the one of the same name in <a href=\"{}/index.html\">{}</a>, which is on the left. Hold the pointer over the right-hand picture to see the left one in its place. Differences are out of 255 for each colour channel.</p>", esc(&rel), esc(&dir_label(o)));
        let _ = writeln!(h, "<table>\n<tr><th>Shot</th><th>Largest difference</th><th>Mean difference</th><th>Pixels that differ</th><th>Their share</th><th>By more than 8</th></tr>");
        for (name, d) in &diffs {
            match d {
                Ok(d) => {
                    let _ = writeln!(
                        h,
                        "<tr{}><td><a href=\"#{n}\">{n}</a></td><td>{}</td><td>{:.6}</td><td>{}</td><td>{:.4}%</td><td>{:.4}%</td></tr>",
                        if d.max > 0 { " class=\"differs\"" } else { "" },
                        d.max,
                        d.mean,
                        d.pixels,
                        d.changed * 100.0,
                        d.visible * 100.0,
                        n = esc(name)
                    );
                }
                Err(e) => {
                    let _ = writeln!(h, "<tr class=\"differs\"><td><a href=\"#{n}\">{n}</a></td><td colspan=\"5\">{}</td></tr>", esc(e), n = esc(name));
                }
            }
        }
        let _ = writeln!(h, "</table>");
        let _ = writeln!(h, "<div class=\"grid pairs\">");
        for ((name, caption), (_, d)) in sheet.shots.iter().zip(&diffs) {
            let file = format!("{}.png", esc(name));
            let theirs = format!("{}/{file}", esc(&rel));
            let num = match d {
                Ok(d) if d.max == 0 => "<span class=\"num\">the same, pixel for pixel</span>".to_string(),
                Ok(d) => format!(
                    "<span class=\"num differs\">largest difference {}, mean {:.6}, {} pixels differ ({:.4}%) &middot; <a class=\"diff\" href=\"diff/{file}\">the difference, 16 times over</a></span>",
                    d.max,
                    d.mean,
                    d.pixels,
                    d.changed * 100.0
                ),
                Err(e) => format!("<span class=\"num differs\">{}</span>", esc(e)),
            };
            let _ = writeln!(h, "<figure id=\"{n}\">\n<figcaption><b>{n}</b><span>{}</span>{num}</figcaption>\n<div class=\"pair\">", esc(caption), n = esc(name));
            let has_theirs = o.join(format!("{name}.png")).exists();
            let has_ours = dir.join(format!("{name}.png")).exists();
            if has_theirs {
                let _ = writeln!(h, "<a href=\"{theirs}\"><img src=\"{theirs}\" loading=\"lazy\" alt=\"{n} in {o}\"><span class=\"tag\">{o}</span></a>", n = esc(name), o = esc(&dir_label(o)));
            } else {
                let _ = writeln!(h, "<div class=\"missing\">not in {}</div>", esc(&dir_label(o)));
            }
            if has_ours {
                let under = if has_theirs { format!("<img class=\"under\" src=\"{theirs}\" loading=\"lazy\" alt=\"\">") } else { String::new() };
                let _ = writeln!(h, "<a class=\"flip\" href=\"{file}\"><span class=\"stack\">{under}<img class=\"over\" src=\"{file}\" loading=\"lazy\" alt=\"{n} in {d}\"></span><span class=\"tag\">{d}</span></a>", n = esc(name), d = esc(&dir_label(dir)));
            } else {
                let _ = writeln!(h, "<div class=\"missing\">not written</div>");
            }
            let _ = writeln!(h, "</div>\n</figure>");
        }
        let _ = writeln!(h, "</div>");
    } else {
        let _ = writeln!(h, "<div class=\"grid\">");
        for (name, caption) in &sheet.shots {
            let file = format!("{}.png", esc(name));
            let _ = writeln!(h, "<figure id=\"{n}\">\n<figcaption><b>{n}</b><span>{}</span></figcaption>", esc(caption), n = esc(name));
            if dir.join(format!("{name}.png")).exists() {
                let _ = writeln!(h, "<a href=\"{file}\"><img src=\"{file}\" loading=\"lazy\" alt=\"{}\"></a>", esc(name));
            } else {
                let _ = writeln!(h, "<div class=\"missing\">not written</div>");
            }
            let _ = writeln!(h, "</figure>");
        }
        let _ = writeln!(h, "</div>");
    }
    let _ = writeln!(h, "</body>\n</html>");
    let path = dir.join("index.html");
    match std::fs::write(&path, h) {
        Ok(()) => eprintln!("wrote {}", path.display()),
        Err(e) => eprintln!("cannot write {}: {e}", path.display()),
    }
    diffs
}

/// Print what `write_sheet` found, a line a shot.
pub fn print_diffs(diffs: &[(String, Result<Diff, String>)]) {
    if diffs.is_empty() {
        return;
    }
    println!("{:<22} {:>8} {:>10} {:>10} {:>10} {:>10}", "shot", "largest", "mean", "pixels", "share", "over 8");
    for (name, d) in diffs {
        match d {
            Ok(d) => println!("{:<22} {:>8} {:>10.6} {:>10} {:>9.4}% {:>9.4}%", name, d.max, d.mean, d.pixels, d.changed * 100.0, d.visible * 100.0),
            Err(e) => println!("{name:<22} {e}"),
        }
    }
}

/// `--compare-shots A B`: compare two sets already shot, B beside A, writing
/// B's sheet anew. True if every shot is the same, pixel for pixel.
pub fn compare_sets(a: &str, b: &str) -> bool {
    let mut sheet = read_sheet(b);
    if sheet.shots.is_empty() {
        eprintln!("no shots in {b}");
        return false;
    }
    sheet.compare = Some(a.to_string());
    let diffs = write_sheet(&sheet);
    print_diffs(&diffs);
    diffs.iter().all(|(_, d)| matches!(d, Ok(d) if d.max == 0))
}
